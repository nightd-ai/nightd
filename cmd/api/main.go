// Package main provides the nightd API control plane CLI.
package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:          "api",
	Short:        "nightd API",
	Long:         "The nightd API manages Omnigent servers and routing.",
	SilenceUsage: true,
	Run: func(cmd *cobra.Command, _ []string) {
		_, _ = fmt.Fprintln(cmd.OutOrStdout(), "nightd API - use 'api start' to run")
	},
}

var startCmd = &cobra.Command{
	Use:   "start",
	Short: "Start the API",
	Run: func(cmd *cobra.Command, _ []string) {
		_, _ = fmt.Fprintln(cmd.OutOrStdout(), "nightd API started")
	},
}

func init() {
	rootCmd.AddCommand(startCmd)
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		_, _ = fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
