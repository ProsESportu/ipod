package main

import (
	"fmt"
	"io"
	"io/ioutil"
	"time"

	"os"

	"github.com/davecgh/go-spew/spew"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli"

	"github.com/oandrew/ipod"
	"github.com/oandrew/ipod/hid"
	audio "github.com/oandrew/ipod/lingo-audio"
	dispremote "github.com/oandrew/ipod/lingo-dispremote"
	extremote "github.com/oandrew/ipod/lingo-extremote"
	"github.com/oandrew/ipod/lingo-extremote/bluez"
	general "github.com/oandrew/ipod/lingo-general"
	_ "github.com/oandrew/ipod/lingo-simpleremote"
	"github.com/oandrew/ipod/trace"
)

var log = logrus.StandardLogger()

func openDevice(path string) (*os.File, error) {
	f, err := os.OpenFile(path, os.O_RDWR, os.ModePerm)
	if err != nil {
		return nil, err
	}
	stat, _ := f.Stat()
	if stat.Mode()&os.ModeCharDevice != os.ModeCharDevice {
		return nil, fmt.Errorf("not a char device")
	}
	return f, nil
}

func openTraceFile(path string) (*os.File, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	return f, nil
}

func newTraceFile(path string) (*os.File, error) {
	return os.OpenFile(path, os.O_RDWR|os.O_CREATE, os.ModePerm)
}

type UsageError struct {
	error
}

var hidReportDefs = hid.DefaultReportDefs

func main() {
	logOut := os.Stdout
	log.Formatter = &TextFormatter{
		Colored: checkIfTerminal(logOut),
	}

	log.Out = logOut

	spew.Config.DisablePointerAddresses = true

	app := cli.NewApp()
	app.Name = "ipod"
	app.Authors = []cli.Author{
		cli.Author{
			Name: "Andrew Onyshchuk",
		},
	}
	app.Usage = "ipod accessory protocol client"
	app.HideVersion = true

	app.ErrWriter = os.Stderr
	app.Flags = []cli.Flag{
		cli.BoolFlag{
			Name:  "debug, d",
			Usage: "verbose logging",
		},
		cli.BoolFlag{
			Name:  "legacy, l",
			Usage: "use legacy hid descriptor",
		},
	}

	app.ExitErrHandler = func(c *cli.Context, err error) {
		if err != nil {
			if _, ok := err.(UsageError); ok {
				fmt.Fprintf(c.App.ErrWriter, "usage error: %v\n\n", err)
				cli.ShowCommandHelp(c, c.Command.Name)
			} else {
				fmt.Fprintf(c.App.ErrWriter, "error: %v\n\n", err)
			}
			os.Exit(1)
		}
	}

	app.Before = func(c *cli.Context) error {
		if c.GlobalBool("debug") {
			log.SetLevel(logrus.DebugLevel)
		}

		if c.GlobalBool("legacy") {
			hidReportDefs = hid.LegacyReportDefs
		}

		return nil
	}

	app.Commands = []cli.Command{
		{
			Name:  "lingos",
			Usage: "print all lingos/commands/ids",
			Action: func(c *cli.Context) error {
				fmt.Println("Registered lingos:")
				fmt.Println(ipod.DumpLingos())
				return nil
			},
		},
		{
			Name:      "serve",
			Aliases:   []string{"s"},
			ArgsUsage: "<dev>",
			Usage:     "respond to requests from a char device i.e. /dev/iap0",
			Flags: []cli.Flag{
				cli.StringFlag{
					Name:  "write-trace, w",
					Usage: "Write trace to a `file`",
				},
				cli.StringFlag{
					Name:  "bluealsa-aplay",
					Usage: "bluealsa-aplay executable `path`",
					Value: defaultBlueALSAAPlay,
				},
				cli.StringSliceFlag{
					Name:  "bluealsa-aplay-arg",
					Usage: "extra argument to pass to bluealsa-aplay",
				},
			},
			Action: func(c *cli.Context) error {
				path := c.Args().First()
				if path == "" {
					return UsageError{fmt.Errorf("device path is missing")}
				}
				ensureExtRemote()

				f, err := openDevice(path)
				le := log.WithField("path", path)
				if err != nil {
					le.WithError(err).Errorf("could not open the device")
					return err
				}
				le.Info("device opened")

				bluealsa := newBlueALSAPlayer(
					c.String("bluealsa-aplay"),
					c.StringSlice("bluealsa-aplay-arg"),
					nil,
					log.Out,
					c.App.ErrWriter,
				)

				var rw io.ReadWriter = f
				if tracePath := c.String("write-trace"); tracePath != "" {
					traceFile, err := newTraceFile(tracePath)
					le := log.WithField("path", tracePath)
					if err != nil {
						le.WithError(err).Errorf("could not create a trace file")
						return err
					}
					le.Warningf("writing trace")
					rw = trace.NewTracer(traceFile, f)
				}

				reportR, reportW := hid.NewReportReader(rw), hid.NewReportWriter(rw)
				frameTransport := hid.NewTransport(reportR, reportW, hidReportDefs)
				frameErr := make(chan error, 1)
				stopFrames := make(chan struct{})
				go func() {
					frameErr <- processFramesWithOptions(frameTransport, frameLoopOptions{
						onPlaybackInitialized: bluealsa.Start,
						stop:                  stopFrames,
					})
				}()

				select {
				case err := <-frameErr:
					select {
					case bluealsaErr := <-bluealsa.Err():
						bluealsa.Stop()
						if err != nil {
							return err
						}
						return bluealsaErr
					default:
					}
					if stopErr := bluealsa.Stop(); stopErr != nil {
						return stopErr
					}
					return err
				case err := <-bluealsa.Err():
					close(stopFrames)
					f.Close()
					<-frameErr
					bluealsa.Stop()
					return err
				}
			},
		},
		{
			Name:    "replay",
			Aliases: []string{"r"},
			Usage:   "respond to requests from a trace file",
			Action: func(c *cli.Context) error {
				path := c.Args().First()
				if path == "" {
					return UsageError{cli.NewExitError("trace file path is missing", 1)}
				}

				f, err := openTraceFile(path)
				le := log.WithField("path", path)
				if err != nil {
					le.WithError(err).Errorf("could not open the trace file")
					return err
				}
				le.Warningf("trace file opened")

				tr := trace.NewReader(f)
				tdr := trace.NewTraceDirReader(tr, trace.DirIn)
				reportR, reportW := hid.NewReportReader(tdr), hid.NewReportWriter(ioutil.Discard)
				frameTransport := hid.NewTransport(reportR, reportW, hidReportDefs)
				processFrames(frameTransport)
				return nil
			},
		},
		{
			Name:    "view",
			Aliases: []string{"v"},
			Usage:   "view a trace file",
			Action: func(c *cli.Context) error {
				path := c.Args().First()
				if path == "" {
					return UsageError{cli.NewExitError("trace file path is missing", 1)}
				}

				f, err := openTraceFile(path)
				le := log.WithField("path", path)
				if err != nil {
					le.WithError(err).Errorf("could not open the trace file")
					return err
				}
				le.Warningf("trace file opened")
				tr := trace.NewReader(f)
				dumpTrace(tr)
				return nil
			},
		},
		{
			Name: "send",
			Flags: []cli.Flag{
				cli.StringFlag{
					Name:  "write-trace, w",
					Usage: "Write trace to a `file`",
				},
			},
			Usage: "acc mode / send accessory requests from a trace file",
			Action: func(c *cli.Context) error {
				path := c.Args().Get(0)
				if path == "" {
					return UsageError{fmt.Errorf("device path is missing")}
				}
				tpath := c.Args().Get(1)
				if tpath == "" {
					return UsageError{cli.NewExitError("trace file path is missing", 1)}
				}
				ensureExtRemote()

				f, err := openDevice(path)
				le := log.WithField("path", path)
				if err != nil {
					le.WithError(err).Errorf("could not open the device")
					return err
				}
				le.Info("device opened")

				tf, err := openTraceFile(tpath)
				tle := log.WithField("path", tpath)
				if err != nil {
					tle.WithError(err).Errorf("could not open the trace file")
					return err
				}
				tle.Warningf("trace file opened")

				tr := trace.NewReader(tf)
				tdr := trace.NewTraceDirReader(tr, trace.DirIn)

				var rw io.ReadWriter = f
				if tracePath := c.String("write-trace"); tracePath != "" {
					traceFile, err := newTraceFile(tracePath)
					le := log.WithField("path", tracePath)
					if err != nil {
						le.WithError(err).Errorf("could not create a trace file")
						return err
					}
					le.Warningf("writing trace")
					rw = trace.NewTracer(traceFile, f)
				}
				reportR, reportW := hid.NewReportReader(rw), hid.NewReportWriter(rw)
				dummyW := hid.NewReportWriter(ioutil.Discard)
				traceR := hid.NewReportReader(tdr)

				frameTransport := hid.NewTransport(reportR, dummyW, hidReportDefs)

				go processFrames(frameTransport)

				for {
					report, err := traceR.ReadReport()
					if err != nil {
						break
					}

					reportW.WriteReport(report)
					log.Infof("writing report\n%s", spew.Sdump(report))

					time.Sleep(1000 * time.Millisecond)
				}

				select {}

				return nil
			},
		},
	}

	app.Setup()
	app.Run(os.Args)

}

func logFrame(frame []byte, err error, msg string) {
	le := FrameLogEntry(logrus.NewEntry(log), frame)
	if err != nil {
		le.WithError(err).Errorf(msg)
		return
	}
	le.Infof(msg)
	if log.Level == logrus.DebugLevel {
		spew.Fdump(log.Out, frame)
	}

}

func logPacket(pkt []byte, err error, msg string) {
	//le := PacketLogEntry(logrus.NewEntry(log), frame)
	le := log.WithField("len", len(pkt))
	if err != nil {
		le.WithError(err).Errorf(msg)
		return
	}
	le.Infof(msg)
	if log.Level == logrus.DebugLevel {
		spew.Fdump(log.Out, pkt)
	}
}

func logCmd(cmd *ipod.Command, err error, msg string) {
	le := CommandLogEntry(logrus.NewEntry(log), cmd)
	if err != nil {
		le.WithError(err).Errorf(msg)
		return
	}
	le.Infof(msg)
	if log.Level == logrus.DebugLevel {
		spew.Fdump(log.Out, cmd)
	}

}

func processFrames(frameTransport ipod.FrameReadWriter) {
	processFramesWithOptions(frameTransport, frameLoopOptions{})
}

type frameLoopOptions struct {
	onPlaybackInitialized func() error
	stop                  <-chan struct{}
}

func processFramesWithOptions(frameTransport ipod.FrameReadWriter, opts frameLoopOptions) error {
	serde := ipod.CommandSerde{}
	playbackInitialized := false

	for {
		inFrame, err := frameTransport.ReadFrame()
		if err == io.EOF {
			break
		}
		logFrame(inFrame, err, "<< FRAME")
		if err != nil {
			select {
			case <-opts.stop:
				return err
			default:
			}
			continue
		}

		packetReader := ipod.NewPacketReader(inFrame)
		inCmdBuf := ipod.CmdBuffer{}
		for {
			inPacket, err := packetReader.ReadPacket()
			if err == io.EOF {
				break
			}
			logPacket(inPacket, err, "<< PACKET")
			if err != nil {
				continue
			}

			inCmd, err := serde.UnmarshalCmd(inPacket)
			logCmd(inCmd, err, "<< CMD")
			inCmdBuf.WriteCommand(inCmd)
		}

		outCmdBuf := ipod.CmdBuffer{}
		shouldInitPlayback := false
		for i := range inCmdBuf.Commands {
			//todo: check return error
			result := handlePacketWithResult(&outCmdBuf, inCmdBuf.Commands[i])
			if result.playbackInitialized {
				shouldInitPlayback = true
			}
		}

		writeErr := false
		for i := range outCmdBuf.Commands {
			outCmd := outCmdBuf.Commands[i]
			logCmd(outCmd, nil, ">> CMD")

			outPacket, err := serde.MarshalCmd(outCmd)
			logPacket(outPacket, err, ">> PACKET")

			packetWriter := ipod.NewPacketWriter()
			packetWriter.WritePacket(outPacket)
			outFrame := packetWriter.Bytes()
			outFrameErr := frameTransport.WriteFrame(outFrame)
			logFrame(outFrame, outFrameErr, ">> FRAME")
			if outFrameErr != nil {
				writeErr = true
			}
		}

		if shouldInitPlayback && !playbackInitialized && !writeErr {
			playbackInitialized = true
			if opts.onPlaybackInitialized != nil {
				if err := opts.onPlaybackInitialized(); err != nil {
					return err
				}
			}
		}

	}
	log.Warnf("EOF")
	return nil
}

var devGeneral = &DevGeneral{}
var devExtRemote extremote.DeviceExtRemote

func ensureExtRemote() {
	if devExtRemote == nil {
		devExtRemote = initExtRemote()
	}
}

// initExtRemote constructs a BlueZ-backed extremote device and blocks until
// BlueZ reports a connected Bluetooth device.
func initExtRemote() extremote.DeviceExtRemote {
	for {
		c, err := bluez.NewClient()
		if err != nil {
			log.WithError(err).Warn("bluez: MediaPlayer1 client unavailable; retrying")
			time.Sleep(5 * time.Second)
			continue
		}
		if !c.DeviceConnected() {
			log.Warn("bluez: waiting for a connected Bluetooth device")
		}
		c.WaitDeviceConnected()
		return c
	}
}

type handlePacketResult struct {
	playbackInitialized bool
}

func handlePacket(cmdWriter ipod.CommandWriter, cmd *ipod.Command) {
	handlePacketWithResult(cmdWriter, cmd)
}

func handlePacketWithResult(cmdWriter ipod.CommandWriter, cmd *ipod.Command) handlePacketResult {
	var result handlePacketResult
	switch cmd.ID.LingoID() {
	case ipod.LingoGeneralID:
		if auth, ok := cmd.Payload.(*general.RetDevAuthenticationInfo); ok {
			if auth.Major >= 2 && auth.CertCurrentSection >= auth.CertMaxSection || auth.Major < 2 {
				audio.Start(cmdWriter)
			}
		}
		general.HandleGeneral(cmd, cmdWriter, devGeneral)

	case ipod.LingoSimpleRemoteID:
		//todo
		log.Warn("Lingo SimpleRemote is not supported yet")
	case ipod.LingoDisplayRemoteID:
		dispremote.HandleDispRemote(cmd, cmdWriter, nil)
	case ipod.LingoExtRemoteID:
		extremote.HandleExtRemote(cmd, cmdWriter, devExtRemote)
	case ipod.LingoDigitalAudioID:
		if _, ok := cmd.Payload.(*audio.RetAccSampleRateCaps); ok {
			result.playbackInitialized = true
		}
		audio.HandleAudio(cmd, cmdWriter, nil)
	}
	return result
}
func dirPrefix(dir trace.Dir, text string) string {
	switch dir {
	case trace.DirIn:
		return "<< " + text
	case trace.DirOut:
		return ">> " + text
	default:
		return "?? " + text
	}
}
func dumpTrace(tr *trace.Reader) {
	q := trace.Queue{}
	for {
		var msg trace.Msg
		err := tr.ReadMsg(&msg)
		if err == io.EOF {
			break
		}
		if err != nil {
			log.Fatal(err)
		}
		q.Enqueue(&msg)
	}

	serde := ipod.CommandSerde{}

	for {
		head := q.Head()
		if head == nil {
			break
		}
		dir := head.Dir
		tdr := trace.NewQueueDirReader(&q, dir)
		d := hid.NewDecoder(hid.NewReportReader(tdr), hidReportDefs)

		frame, err := d.ReadFrame()
		if err == io.EOF {
			break
		}
		logFrame(frame, err, dirPrefix(dir, "FRAME"))
		if err != nil {
			continue
		}

		packetReader := ipod.NewPacketReader(frame)
		for {
			packet, err := packetReader.ReadPacket()
			if err == io.EOF {
				break
			}
			logPacket(packet, err, dirPrefix(dir, "PACKET"))
			if err != nil {
				continue
			}

			cmd, err := serde.UnmarshalCmd(packet)
			logCmd(cmd, err, dirPrefix(dir, "CMD"))
		}
	}
	log.Warnf("EOF")
}
