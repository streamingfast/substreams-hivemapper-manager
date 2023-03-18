package main

import (
	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:          "fleet-split-manager",
	Short:        "A tool for fleets to manage payout splits",
	SilenceUsage: true,
}

func init() {

}
