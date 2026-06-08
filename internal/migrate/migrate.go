package migrate

import (
	"database/sql"
	"fmt"

	"github.com/golang-migrate/migrate/v4"
	pgx "github.com/golang-migrate/migrate/v4/database/pgx/v5"
	"github.com/golang-migrate/migrate/v4/source/iofs"
	"github.com/nightd-ai/nightd/db"
)

func RunUp(sqlDB *sql.DB) error {
	source, err := iofs.New(db.Migrations, "migrations")
	if err != nil {
		return fmt.Errorf("failed to create migration source: %w", err)
	}

	driver, err := pgx.WithInstance(sqlDB, &pgx.Config{})
	if err != nil {
		return fmt.Errorf("failed to create migrate driver: %w", err)
	}

	m, err := migrate.NewWithInstance("iofs", source, "pgx5", driver)
	if err != nil {
		return fmt.Errorf("failed to create migrate instance: %w", err)
	}
	defer func() { _, _ = m.Close() }()

	if err := m.Up(); err != nil && err != migrate.ErrNoChange {
		return fmt.Errorf("failed to run migrations: %w", err)
	}

	return nil
}

func RunDown(sqlDB *sql.DB) error {
	source, err := iofs.New(db.Migrations, "migrations")
	if err != nil {
		return fmt.Errorf("failed to create migration source: %w", err)
	}

	driver, err := pgx.WithInstance(sqlDB, &pgx.Config{})
	if err != nil {
		return fmt.Errorf("failed to create migrate driver: %w", err)
	}

	m, err := migrate.NewWithInstance("iofs", source, "pgx5", driver)
	if err != nil {
		return fmt.Errorf("failed to create migrate instance: %w", err)
	}
	defer func() { _, _ = m.Close() }()

	if err := m.Down(); err != nil && err != migrate.ErrNoChange {
		return fmt.Errorf("failed to run down migration: %w", err)
	}

	return nil
}

func GetVersion(sqlDB *sql.DB) (version uint, dirty bool, err error) {
	source, err := iofs.New(db.Migrations, "migrations")
	if err != nil {
		return 0, false, fmt.Errorf("failed to create migration source: %w", err)
	}

	driver, err := pgx.WithInstance(sqlDB, &pgx.Config{})
	if err != nil {
		return 0, false, fmt.Errorf("failed to create migrate driver: %w", err)
	}

	m, err := migrate.NewWithInstance("iofs", source, "pgx5", driver)
	if err != nil {
		return 0, false, fmt.Errorf("failed to create migrate instance: %w", err)
	}
	defer func() { _, _ = m.Close() }()

	version, dirty, err = m.Version()
	if err != nil && err != migrate.ErrNilVersion {
		return 0, false, fmt.Errorf("failed to get version: %w", err)
	}

	return version, dirty, nil
}
