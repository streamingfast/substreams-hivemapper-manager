package main

import (
	"encoding/json"
	"fmt"
	"github.com/spf13/cobra"
	"os/exec"
	"strconv"
	"strings"
)

var payoutsCmd = &cobra.Command{
	Use:          "payouts <json-file>",
	Short:        "get all hivemapper fleet payouts",
	RunE:         runPayoutsCmd,
	Args:         cobra.ExactArgs(0),
	SilenceUsage: true,
}

func init() {
	rootCmd.AddCommand(payoutsCmd)
}

type PayoutsData struct {
	Payouts []struct {
		TransactionID string `json:"transactionId"`
		To            string `json:"to"`
		Amount        string `json:"amount"`
	} `json:"payouts"`
}

type Payouts struct {
	Module string      `json:"@module"`
	Block  int         `json:"@block"`
	Type   string      `json:"@type"`
	Data   PayoutsData `json:"@data"`
}

type Referrals struct {
	Referrals []Referral `json:"referrals"`
}

type Referral struct {
	Recruited string `json:"gotReferred"`
	Recruiter string `json:"isOwed"`
}

func runPayoutsCmd(cmd *cobra.Command, args []string) error {
	_ = "3Pa4DNHKyEPJ5YQPaQBRDggstgmd89Zhr4yVMndo6T4C"
	//referralsBytes = []byte(args[0])

	startBlock := strconv.Itoa(180279567)
	substreamsOut, err := exec.Command("substreams", "run", "-e", "mainnet.sol.streamingfast.io:443", "-o", "json", "./fleet-substream/substreams.yaml", "map_payouts", "-s", startBlock, "-t", "+2", "-i").Output()
	if err != nil {
		return fmt.Errorf("running substreams: %w", err)
	}

	amounts, txns, receivers, _, err := readSubstreamsOutput(substreamsOut)
	if err != nil {
		return fmt.Errorf("digesting substream output: %w", err)
	}

	for i, amount := range amounts {
		fmt.Printf("amount: %s, to: %s, on: %s\n", amount, receivers[i], txns[i])
	}

	//fmt.Printf("%s received a fleet payout split for: %s\n%s received a fleet payout split for: %s\nThe split was %v")
	return nil
}

func readSubstreamsOutput(response []byte) (amounts, txns, addresses []string, duplicateIndexes []int, err error) {
	substreamsOut := response[20 : len(response)-10]

	blockByBlock := strings.Split(string(substreamsOut), "\n\n")
	for _, block := range blockByBlock {
		blockBytes := []byte(block)

		var substreamsResponse Payouts
		err := json.Unmarshal(blockBytes, &substreamsResponse)
		if err != nil {
			return nil, nil, nil, nil, fmt.Errorf("unmarshaling response: %w", err)
		}

		curHash := ""
		for i, payout := range substreamsResponse.Data.Payouts {

			txns = append(txns, payout.TransactionID)
			amounts = append(amounts, payout.Amount)
			addresses = append(addresses, payout.To)

			if payout.TransactionID == curHash {
				duplicateIndexes = append(duplicateIndexes, i)
			}
			curHash = payout.TransactionID
		}
	}

	return amounts, txns, addresses, duplicateIndexes, nil
}
