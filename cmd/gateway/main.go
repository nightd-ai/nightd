// Package main provides the nightd gateway proxy CLI.
package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:          "gateway",
	Short:        "nightd gateway",
	Long:         "The nightd gateway proxies authenticated requests to the correct Omnigent server for a workspace.",
	SilenceUsage: true,
	Run: func(cmd *cobra.Command, _ []string) {
		_, _ = fmt.Fprintln(cmd.OutOrStdout(), "nightd gateway - use 'gateway start' to run")
	},
}

var startCmd = &cobra.Command{
	Use:   "start",
	Short: "Start the gateway",
	Run: func(cmd *cobra.Command, _ []string) {
		_, _ = fmt.Fprintln(cmd.OutOrStdout(), "nightd gateway started")
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
