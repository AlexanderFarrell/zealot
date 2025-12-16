/*
Copyright © 2025 NAME HERE <EMAIL ADDRESS>

*/
package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

// itemCmd represents the item command
var itemCmd = &cobra.Command{
	Use:   "item",
	Short: "Item description",
	Long: ``,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("item called")
	},
}

func init() {
	rootCmd.AddCommand(itemCmd)

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// itemCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// itemCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
