package scheduler

import (
	"context"
	"log"
	"os"
	"testing"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/nightd-ai/nightd/internal/migrate"
)

var testPool *pgxpool.Pool

func TestMain(m *testing.M) {
	if err := migrate.RunUp(); err != nil {
		log.Fatal("failed to run migrations:", err)
	}

	dsn := os.Getenv("DATABASE_URL")
	if dsn == "" {
		log.Fatal("DATABASE_URL is not set")
	}

	var err error
	testPool, err = pgxpool.New(context.Background(), dsn)
	if err != nil {
		log.Fatal("failed to create test pool:", err)
	}

	code := m.Run()
	testPool.Close()
	os.Exit(code)
}

func cleanupTasks(t *testing.T) {
	t.Helper()
	_, err := testPool.Exec(context.Background(), "TRUNCATE tasks")
	if err != nil {
		t.Fatalf("failed to truncate tasks: %v", err)
	}
}

func TestSchedulerProcessesTasks(t *testing.T) {
	cleanupTasks(t)

	var taskID string
	err := testPool.QueryRow(context.Background(), `
		INSERT INTO tasks (status) VALUES ('pending') RETURNING id
	`).Scan(&taskID)
	if err != nil {
		t.Fatalf("failed to insert task: %v", err)
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	sched := New(testPool)
	go func() {
		if err := sched.Run(ctx); err != nil && err != context.Canceled {
			t.Errorf("scheduler error: %v", err)
		}
	}()

	time.Sleep(5 * time.Second)

	var status string
	err = testPool.QueryRow(context.Background(), `SELECT status FROM tasks WHERE id = $1`, taskID).Scan(&status)
	if err != nil {
		t.Fatalf("failed to query task status: %v", err)
	}

	if status != "done" {
		t.Fatalf("expected task status to be 'done', got %q", status)
	}

	cancel()
}
