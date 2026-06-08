// Package db embeds database migration files into the binary.
package db

import "embed"

// Migrations contains the embedded database migration files.
//
//go:embed migrations/*.sql
var Migrations embed.FS
