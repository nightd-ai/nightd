// Package scheduler provides a task queue poller and processor.
package scheduler

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

// Scheduler polls the task queue and processes tasks in parallel.
type Scheduler struct {
	db *pgxpool.Pool
}

// New creates a new Scheduler with the given database pool.
func New(db *pgxpool.Pool) *Scheduler {
	return &Scheduler{db: db}
}

// Run starts polling the task queue and processing tasks until the context is cancelled.
func (s *Scheduler) Run(ctx context.Context) error {
	ticker := time.NewTicker(2 * time.Second)
	defer ticker.Stop()

	if err := s.processBatch(ctx); err != nil {
		fmt.Printf("error processing initial batch: %v\n", err)
	}

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-ticker.C:
			if err := s.processBatch(ctx); err != nil {
				fmt.Printf("error processing batch: %v\n", err)
			}
		}
	}
}

func (s *Scheduler) processBatch(ctx context.Context) error {
	tx, err := s.db.BeginTx(ctx, pgx.TxOptions{})
	if err != nil {
		return fmt.Errorf("begin transaction: %w", err)
	}
	defer func() {
		_ = tx.Rollback(ctx)
	}()

	rows, err := tx.Query(ctx, `
		SELECT id FROM tasks WHERE status = 'pending' ORDER BY created_at ASC LIMIT 10 FOR UPDATE SKIP LOCKED
	`)
	if err != nil {
		return fmt.Errorf("query pending tasks: %w", err)
	}

	var ids []string
	for rows.Next() {
		var id string
		if err := rows.Scan(&id); err != nil {
			rows.Close()
			return fmt.Errorf("scan task id: %w", err)
		}
		ids = append(ids, id)
	}
	rows.Close()

	if len(ids) == 0 {
		return nil
	}

	for _, id := range ids {
		_, err := tx.Exec(ctx, `UPDATE tasks SET status = 'in_progress' WHERE id = $1`, id)
		if err != nil {
			return fmt.Errorf("update task status to in_progress: %w", err)
		}
	}

	if err := tx.Commit(ctx); err != nil {
		return fmt.Errorf("commit transaction: %w", err)
	}

	var wg sync.WaitGroup
	for _, id := range ids {
		wg.Add(1)
		go func(taskID string) {
			defer wg.Done()
			s.processTask(ctx, taskID)
		}(id)
	}
	wg.Wait()

	return nil
}

func (s *Scheduler) processTask(ctx context.Context, id string) {
	fmt.Printf("Processing task: %s\n", id)

	_, err := s.db.Exec(ctx, `UPDATE tasks SET status = 'done' WHERE id = $1`, id)
	if err != nil {
		fmt.Printf("error updating task %s to done: %v\n", id, err)
		_, err = s.db.Exec(ctx, `UPDATE tasks SET status = 'error' WHERE id = $1`, id)
		if err != nil {
			fmt.Printf("error updating task %s to error: %v\n", id, err)
		}
	}
}
