package main

import (
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"sync"
	"syscall"
	"time"
)

const defaultBlueALSAAPlay = "bluealsa-aplay"

type blueALSAProcess interface {
	Wait() error
	Signal(os.Signal) error
	Kill() error
}

type blueALSARunner interface {
	Start(name string, args []string, stdout, stderr io.Writer) (blueALSAProcess, error)
}

type execBlueALSARunner struct{}

func (execBlueALSARunner) Start(name string, args []string, stdout, stderr io.Writer) (blueALSAProcess, error) {
	if name == "" {
		return nil, fmt.Errorf("bluealsa: executable path is empty")
	}
	cmd := exec.Command(name, args...)
	cmd.Stdout = stdout
	cmd.Stderr = stderr
	if err := cmd.Start(); err != nil {
		return nil, fmt.Errorf("bluealsa: start %s: %w", name, err)
	}
	return &execBlueALSAProcess{cmd: cmd}, nil
}

type execBlueALSAProcess struct {
	cmd *exec.Cmd
}

func (p *execBlueALSAProcess) Wait() error {
	return p.cmd.Wait()
}

func (p *execBlueALSAProcess) Signal(sig os.Signal) error {
	if p.cmd.Process == nil {
		return os.ErrProcessDone
	}
	return p.cmd.Process.Signal(sig)
}

func (p *execBlueALSAProcess) Kill() error {
	if p.cmd.Process == nil {
		return os.ErrProcessDone
	}
	return p.cmd.Process.Kill()
}

type blueALSAProcessState struct {
	proc blueALSAProcess
	done chan struct{}
	err  error
}

type blueALSAPlayer struct {
	command     string
	args        []string
	runner      blueALSARunner
	stdout      io.Writer
	stderr      io.Writer
	stopTimeout time.Duration

	mu       sync.Mutex
	state    *blueALSAProcessState
	stopping bool
	errCh    chan error
}

func newBlueALSAPlayer(command string, args []string, runner blueALSARunner, stdout, stderr io.Writer) *blueALSAPlayer {
	if command == "" {
		command = defaultBlueALSAAPlay
	}
	if runner == nil {
		runner = execBlueALSARunner{}
	}
	if stdout == nil {
		stdout = os.Stdout
	}
	if stderr == nil {
		stderr = os.Stderr
	}
	return &blueALSAPlayer{
		command:     command,
		args:        append([]string(nil), args...),
		runner:      runner,
		stdout:      stdout,
		stderr:      stderr,
		stopTimeout: 5 * time.Second,
		errCh:       make(chan error, 1),
	}
}

func (p *blueALSAPlayer) Err() <-chan error {
	return p.errCh
}

func (p *blueALSAPlayer) Start() error {
	p.mu.Lock()
	if p.state != nil {
		p.mu.Unlock()
		return nil
	}

	log.WithField("cmd", p.command).WithField("args", p.args).Info("bluealsa: starting bluealsa-aplay")
	proc, err := p.runner.Start(p.command, p.args, p.stdout, p.stderr)
	if err != nil {
		p.mu.Unlock()
		return err
	}

	state := &blueALSAProcessState{
		proc: proc,
		done: make(chan struct{}),
	}
	p.state = state
	p.stopping = false
	p.mu.Unlock()

	go p.wait(state)
	return nil
}

func (p *blueALSAPlayer) Stop() error {
	p.mu.Lock()
	state := p.state
	if state == nil {
		p.mu.Unlock()
		return nil
	}
	p.stopping = true
	p.mu.Unlock()

	if err := state.proc.Signal(syscall.SIGTERM); err != nil && !errors.Is(err, os.ErrProcessDone) {
		return err
	}

	select {
	case <-state.done:
		return nil
	case <-time.After(p.stopTimeout):
	}

	if err := state.proc.Kill(); err != nil && !errors.Is(err, os.ErrProcessDone) {
		return err
	}
	<-state.done
	return nil
}

func (p *blueALSAPlayer) wait(state *blueALSAProcessState) {
	state.err = state.proc.Wait()

	p.mu.Lock()
	intentional := p.stopping
	if p.state == state {
		p.state = nil
		p.stopping = false
	}
	p.mu.Unlock()

	if !intentional {
		p.reportUnexpectedExit(state.err)
	}
	close(state.done)
}

func (p *blueALSAPlayer) reportUnexpectedExit(err error) {
	if err == nil {
		err = fmt.Errorf("bluealsa: bluealsa-aplay exited")
	} else {
		err = fmt.Errorf("bluealsa: bluealsa-aplay exited: %w", err)
	}
	select {
	case p.errCh <- err:
	default:
	}
}
