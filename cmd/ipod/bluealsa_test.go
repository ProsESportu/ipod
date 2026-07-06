package main

import (
	"errors"
	"io"
	"os"
	"reflect"
	"sync"
	"syscall"
	"testing"
	"time"
)

type fakeBlueALSAStart struct {
	name string
	args []string
}

type fakeBlueALSARunner struct {
	mu     sync.Mutex
	starts []fakeBlueALSAStart
	proc   *fakeBlueALSAProcess
	err    error
}

func (r *fakeBlueALSARunner) Start(name string, args []string, stdout, stderr io.Writer) (blueALSAProcess, error) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.starts = append(r.starts, fakeBlueALSAStart{name: name, args: append([]string(nil), args...)})
	if r.err != nil {
		return nil, r.err
	}
	return r.proc, nil
}

type fakeBlueALSAProcess struct {
	waitCh   chan error
	mu       sync.Mutex
	signals  []os.Signal
	killed   bool
	onSignal func(os.Signal) error
	onKill   func() error
}

func newFakeBlueALSAProcess() *fakeBlueALSAProcess {
	return &fakeBlueALSAProcess{waitCh: make(chan error, 1)}
}

func (p *fakeBlueALSAProcess) Wait() error {
	return <-p.waitCh
}

func (p *fakeBlueALSAProcess) Signal(sig os.Signal) error {
	p.mu.Lock()
	p.signals = append(p.signals, sig)
	onSignal := p.onSignal
	p.mu.Unlock()
	if onSignal != nil {
		return onSignal(sig)
	}
	return nil
}

func (p *fakeBlueALSAProcess) Kill() error {
	p.mu.Lock()
	p.killed = true
	onKill := p.onKill
	p.mu.Unlock()
	if onKill != nil {
		return onKill()
	}
	return nil
}

func (p *fakeBlueALSAProcess) exit(err error) {
	p.waitCh <- err
}

func TestBlueALSAPlayerStartBuildsArgv(t *testing.T) {
	proc := newFakeBlueALSAProcess()
	proc.onSignal = func(os.Signal) error {
		proc.exit(nil)
		return nil
	}
	runner := &fakeBlueALSARunner{proc: proc}
	player := newBlueALSAPlayer("custom-bluealsa-aplay", []string{"--profile-a2dp", "--pcm", "default"}, runner, nil, nil)

	if err := player.Start(); err != nil {
		t.Fatalf("Start: %v", err)
	}
	if err := player.Stop(); err != nil {
		t.Fatalf("Stop: %v", err)
	}

	want := []fakeBlueALSAStart{{
		name: "custom-bluealsa-aplay",
		args: []string{"--profile-a2dp", "--pcm", "default"},
	}}
	if !reflect.DeepEqual(runner.starts, want) {
		t.Fatalf("starts = %#v, want %#v", runner.starts, want)
	}
}

func TestBlueALSAPlayerStartIsIdempotent(t *testing.T) {
	proc := newFakeBlueALSAProcess()
	proc.onSignal = func(os.Signal) error {
		proc.exit(nil)
		return nil
	}
	runner := &fakeBlueALSARunner{proc: proc}
	player := newBlueALSAPlayer(defaultBlueALSAAPlay, nil, runner, nil, nil)

	if err := player.Start(); err != nil {
		t.Fatalf("first Start: %v", err)
	}
	if err := player.Start(); err != nil {
		t.Fatalf("second Start: %v", err)
	}
	if err := player.Stop(); err != nil {
		t.Fatalf("Stop: %v", err)
	}

	if len(runner.starts) != 1 {
		t.Fatalf("start calls = %d, want 1", len(runner.starts))
	}
}

func TestBlueALSAPlayerStartFailure(t *testing.T) {
	startErr := errors.New("missing executable")
	runner := &fakeBlueALSARunner{err: startErr}
	player := newBlueALSAPlayer(defaultBlueALSAAPlay, nil, runner, nil, nil)

	if err := player.Start(); !errors.Is(err, startErr) {
		t.Fatalf("Start error = %v, want %v", err, startErr)
	}
}

func TestBlueALSAPlayerReportsUnexpectedExit(t *testing.T) {
	proc := newFakeBlueALSAProcess()
	runner := &fakeBlueALSARunner{proc: proc}
	player := newBlueALSAPlayer(defaultBlueALSAAPlay, nil, runner, nil, nil)

	if err := player.Start(); err != nil {
		t.Fatalf("Start: %v", err)
	}
	exitErr := errors.New("boom")
	proc.exit(exitErr)

	select {
	case err := <-player.Err():
		if !errors.Is(err, exitErr) {
			t.Fatalf("unexpected exit error = %v, want wrapped %v", err, exitErr)
		}
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for unexpected exit")
	}
}

func TestBlueALSAPlayerStopSuppressesUnexpectedExit(t *testing.T) {
	proc := newFakeBlueALSAProcess()
	proc.onSignal = func(sig os.Signal) error {
		if sig != syscall.SIGTERM {
			t.Fatalf("signal = %v, want SIGTERM", sig)
		}
		proc.exit(errors.New("terminated"))
		return nil
	}
	runner := &fakeBlueALSARunner{proc: proc}
	player := newBlueALSAPlayer(defaultBlueALSAAPlay, nil, runner, nil, nil)

	if err := player.Start(); err != nil {
		t.Fatalf("Start: %v", err)
	}
	if err := player.Stop(); err != nil {
		t.Fatalf("Stop: %v", err)
	}

	select {
	case err := <-player.Err():
		t.Fatalf("unexpected error after Stop: %v", err)
	case <-time.After(20 * time.Millisecond):
	}
}

func TestBlueALSAPlayerStopKillsAfterTimeout(t *testing.T) {
	proc := newFakeBlueALSAProcess()
	proc.onKill = func() error {
		proc.exit(errors.New("killed"))
		return nil
	}
	runner := &fakeBlueALSARunner{proc: proc}
	player := newBlueALSAPlayer(defaultBlueALSAAPlay, nil, runner, nil, nil)
	player.stopTimeout = time.Millisecond

	if err := player.Start(); err != nil {
		t.Fatalf("Start: %v", err)
	}
	if err := player.Stop(); err != nil {
		t.Fatalf("Stop: %v", err)
	}
	if !proc.killed {
		t.Fatal("process was not killed after stop timeout")
	}
}
