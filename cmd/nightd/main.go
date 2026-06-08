package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"sync"
	"syscall"

	"github.com/jackc/pgx/v5/stdlib"
	"github.com/nightd-ai/nightd/internal/db"
	"github.com/nightd-ai/nightd/internal/migrate"
	"github.com/nightd-ai/nightd/internal/scheduler"
	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:   "nightd",
	Short: "A daemon to schedule autonomous coding agents",
	Long:  `nightd is a daemon that schedules and manages autonomous coding agents.`,
}

var startCmd = &cobra.Command{
	Use:   "start",
	Short: "Start the nightd daemon",
	Long:  `Start the nightd daemon to begin scheduling autonomous coding agents.`,
	Run: func(cmd *cobra.Command, args []string) {
		pool, err := db.NewPool(cmd.Context())
		if err != nil {
			fmt.Fprintln(os.Stderr, "failed to create db pool:", err)
			os.Exit(1)
		}
		defer pool.Close()

		noMigrate, _ := cmd.Flags().GetBool("no-migrate")
		if !noMigrate {
			sqlDB := stdlib.OpenDBFromPool(pool)
			if err := migrate.RunUp(sqlDB); err != nil {
				fmt.Fprintln(os.Stderr, "migration failed:", err)
				os.Exit(1)
			}
		}

		sched := scheduler.New(pool)

		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		sigCh := make(chan os.Signal, 1)
		signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

		var wg sync.WaitGroup
		wg.Add(1)
		go func() {
			defer wg.Done()
			if err := sched.Run(ctx); err != nil && err != context.Canceled {
				fmt.Fprintln(os.Stderr, "scheduler error:", err)
			}
		}()

		fmt.Println("nightd daemon started successfully! Good luck with your autonomous coding agents.")

		<-sigCh
		fmt.Println("shutting down...")
		cancel()
		wg.Wait()
		fmt.Println("nightd daemon stopped")
	},
}

var migrateCmd = &cobra.Command{
	Use:   "migrate",
	Short: "Database migration commands",
	Long:  `Commands to manage database migrations.`,
}

var migrateUpCmd = &cobra.Command{
	Use:   "up",
	Short: "Run all pending up migrations",
	Run: func(cmd *cobra.Command, args []string) {
		pool, err := db.NewPool(cmd.Context())
		if err != nil {
			fmt.Fprintln(os.Stderr, "failed to create db pool:", err)
			os.Exit(1)
		}
		defer pool.Close()

		sqlDB := stdlib.OpenDBFromPool(pool)
		if err := migrate.RunUp(sqlDB); err != nil {
			fmt.Fprintln(os.Stderr, "migrate up failed:", err)
			os.Exit(1)
		}
		fmt.Println("migrations up completed successfully")
	},
}

var migrateDownCmd = &cobra.Command{
	Use:   "down",
	Short: "Run one step down migration",
	Run: func(cmd *cobra.Command, args []string) {
		pool, err := db.NewPool(cmd.Context())
		if err != nil {
			fmt.Fprintln(os.Stderr, "failed to create db pool:", err)
			os.Exit(1)
		}
		defer pool.Close()

		sqlDB := stdlib.OpenDBFromPool(pool)
		if err := migrate.RunDown(sqlDB); err != nil {
			fmt.Fprintln(os.Stderr, "migrate down failed:", err)
			os.Exit(1)
		}
		fmt.Println("migrations down completed successfully")
	},
}

var migrateVersionCmd = &cobra.Command{
	Use:   "version",
	Short: "Print current migration version and dirty status",
	Run: func(cmd *cobra.Command, args []string) {
		pool, err := db.NewPool(cmd.Context())
		if err != nil {
			fmt.Fprintln(os.Stderr, "failed to create db pool:", err)
			os.Exit(1)
		}
		defer pool.Close()

		sqlDB := stdlib.OpenDBFromPool(pool)
		version, dirty, err := migrate.GetVersion(sqlDB)
		if err != nil {
			fmt.Fprintln(os.Stderr, "migrate version failed:", err)
			os.Exit(1)
		}
		fmt.Printf("version: %d, dirty: %v\n", version, dirty)
	},
}

func init() {
	startCmd.Flags().Bool("no-migrate", false, "Skip automatic migrations on start")
	rootCmd.AddCommand(startCmd)

	migrateCmd.AddCommand(migrateUpCmd)
	migrateCmd.AddCommand(migrateDownCmd)
	migrateCmd.AddCommand(migrateVersionCmd)
	rootCmd.AddCommand(migrateCmd)
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
