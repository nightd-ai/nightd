// Package main provides the nightd node data plane CLI.
package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:          "node",
	Short:        "nightd node",
	Long:         "The nightd node launches an Omnigent server for a specified workspace.",
	SilenceUsage: true,
	Run: func(cmd *cobra.Command, _ []string) {
		_, _ = fmt.Fprintln(cmd.OutOrStdout(), "nightd node - use 'node start' to run")
	},
}

var startCmd = &cobra.Command{
	Use:   "start",
	Short: "Start the node",
	Run: func(cmd *cobra.Command, _ []string) {
		_, _ = fmt.Fprintln(cmd.OutOrStdout(), "nightd node started")
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
