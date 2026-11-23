// This test is overlaid into the pinned ai-arena service package by verify.sh.
package service

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"testing"

	"github.com/yoskeoka/ai-arena/internal/platform/contract"
	"github.com/yoskeoka/ai-arena/internal/platform/registry"
	"github.com/yoskeoka/ai-arena/internal/platform/replay"
)

func TestReversiReleaseCompetitions(t *testing.T) {
	readBundle := func(variable string) []byte {
		t.Helper()
		data, err := os.ReadFile(os.Getenv(variable))
		if err != nil {
			t.Fatal(err)
		}
		return data
	}
	ctx := t.Context()
	gameBytes := readBundle("REVERSI_GAME_BUNDLE")
	aiBytes := readBundle("REVERSI_AI_BUNDLE")
	bundles, err := NewFilesystemBundleStore(t.TempDir())
	if err != nil {
		t.Fatal(err)
	}
	reg, err := registry.NewWASIOverlay(bundles)
	if err != nil {
		t.Fatal(err)
	}
	admission, err := NewArtifactAdmissionService(bundles, reg)
	if err != nil {
		t.Fatal(err)
	}
	release, err := admission.RegisterGameBundle(ctx, gameBytes)
	if err != nil {
		t.Fatal(err)
	}

	root := repoRoot(t)
	general, err := NewGeneralSubmissionService(root, reg, nil, nil)
	if err != nil {
		t.Fatal(err)
	}
	general.WithBundleStore(bundles)
	queue := NewInMemoryQueueStore()
	dryRun, err := NewLocalDryRunChecker(root)
	if err != nil {
		t.Fatal(err)
	}
	validator, err := NewDefaultAdmissionValidator(reg, dryRun.WithBundleStore(bundles))
	if err != nil {
		t.Fatal(err)
	}
	commands, err := NewCommandService(queue, validator)
	if err != nil {
		t.Fatal(err)
	}
	requests, err := NewMatchRequestService(general, commands, queue, nil)
	if err != nil {
		t.Fatal(err)
	}

	var queued []QueueRecord
	for _, ruleset := range []string{"standard", "standard-xot-large", "standard-xot-small"} {
		registration, err := general.RegisterGame(ctx, GameRegistrationRequest{
			ArtifactID: release.ArtifactID, RulesetVersion: ruleset,
		})
		if err != nil {
			t.Fatalf("register %s: %v", ruleset, err)
		}
		var participants []MatchRequestParticipant
		for _, player := range []string{"black", "white"} {
			ai, err := general.RegisterAIBundle(ctx, AISubmissionRequest{
				GameRegistrationID: registration.RegistrationID, DisplayName: player,
			}, aiBytes)
			if err != nil {
				t.Fatal(err)
			}
			participants = append(participants, MatchRequestParticipant{PlayerID: player, AISubmissionID: ai.AISubmissionID})
		}
		outputDir := t.TempDir()
		if evidenceDir := os.Getenv("REVERSI_MATCH_EVIDENCE_DIR"); evidenceDir != "" {
			outputDir = filepath.Join(evidenceDir, ruleset)
		}
		_, run, err := requests.Create(ctx, MatchRequestCreateRequest{
			RequestID: ruleset, GameRegistrationID: registration.RegistrationID, MatchID: ruleset,
			OutputDir: outputDir, Participants: participants,
		})
		if err != nil {
			t.Fatalf("queue %s: %v", ruleset, err)
		}
		if run.Submission.GameArtifactID != release.ArtifactID || run.Submission.Game.RulesetVersion != ruleset {
			t.Fatalf("queued game identity = %+v", run.Submission)
		}
		queued = append(queued, run)
	}

	invoker, err := NewLocalRunnerInvoker(root, reg, 0)
	if err != nil {
		t.Fatal(err)
	}
	worker, err := NewWorker(queue, invoker.WithBundleStore(bundles), LocalTerminalPersister{})
	if err != nil {
		t.Fatal(err)
	}
	var workers sync.WaitGroup
	for i := range queued {
		workers.Go(func() {
			if _, err := worker.ProcessNext(ctx, fmt.Sprintf("reversi-worker-%d", i)); err != nil {
				t.Errorf("worker %d: %v", i, err)
			}
		})
	}
	workers.Wait()

	for _, original := range queued {
		ruleset := original.Submission.Game.RulesetVersion
		t.Run(ruleset, func(t *testing.T) {
			run, err := queue.Get(ctx, original.Submission.RunID)
			if err != nil {
				t.Fatal(err)
			}
			if run.State != StateCompleted || run.Terminal == nil || run.Terminal.MatchStatus != contract.StatusCompleted {
				t.Fatalf("WASI match did not complete: %+v", run)
			}
			record, err := replay.LoadRecord(run.Terminal.RecordPath)
			if err != nil {
				t.Fatal(err)
			}
			if record.GameArtifactID != release.ArtifactID || record.Game.RulesetVersion != ruleset ||
				record.Snapshot.RulesetVersion != ruleset || record.ExportedSnapshot.RulesetVersion != ruleset {
				t.Fatal("record game identity differs from registered ruleset and artifact")
			}
			openingMoves := 0
			if ruleset != "standard" {
				openingMoves = 8
			}
			if record.PublicReplay == nil || record.PublicReplay.Format != "reversi/replay" || record.PublicReplay.Version != "1" {
				t.Fatal("missing versioned public Reversi replay")
			}
			published, err := os.ReadFile(run.Terminal.PublicReplayPath)
			if err != nil || !bytes.Equal(published, record.PublicReplay.Payload) {
				t.Fatalf("published replay does not match the record: %v", err)
			}
			var payload struct {
				Ruleset string            `json:"ruleset"`
				Opening []json.RawMessage `json:"opening_moves"`
				Turns   []json.RawMessage `json:"turns"`
			}
			if err := json.Unmarshal(record.PublicReplay.Payload, &payload); err != nil {
				t.Fatal(err)
			}
			if payload.Ruleset != ruleset || len(payload.Opening) != openingMoves || len(payload.Turns) == 0 ||
				record.Snapshot.Turn != openingMoves+1+len(payload.Turns) {
				t.Fatalf("unexpected %s replay metadata or turns", ruleset)
			}
			for _, event := range record.EventLog {
				if event.Kind == "turn_requested" {
					if event.Turn != openingMoves+1 || event.PlayerID != "black" {
						t.Fatalf("unexpected first AI request: %+v", event)
					}
					return
				}
			}
			t.Fatal("missing AI turn")
		})
	}
}
