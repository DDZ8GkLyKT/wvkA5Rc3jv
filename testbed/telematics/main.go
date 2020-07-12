package main

import (
	"flag"
	"fmt"
	"os"
)

func main() {
	logCommand := flag.NewFlagSet("log", flag.ExitOnError)
	intervalFlag := logCommand.Uint("interval", 1, "Sets the interval between data points")
	durationFlag := logCommand.Uint("duration", 3600, "Sets the duration of the log")
	nodeListFlag := logCommand.String("nodelist", "nodes.txt", "Sets the path to the node list file")
	dataDirFlag := logCommand.String("datadir", "data", "Sets the path to the directory to hold data")
	numShardsFlag := logCommand.Uint("num_shards", 1, "Sets the total number of shards")
	targetFileFlag := logCommand.String("targetfile", "godump.txt", "Output dump file")
	grafanaFlag := logCommand.Bool("grafana", false, "Record extra data for displaying on Grafana")
	displayFlag := logCommand.Bool("display", false, "Record extra data for displaying on Grafana")

	plotCommand := flag.NewFlagSet("plot", flag.ExitOnError)
	plotNodeListFlag := plotCommand.String("nodelist", "nodes.txt", "Sets the path to the node list file")
	plotDataDirFlag := plotCommand.String("datadir", "data", "Sets the path to the directory holding RRD files")
	plotContentFlag := plotCommand.String("content", "txrate", "Sets the content to plot, possible values are txrate, blockdelay, queue, mining, confirm")
	plotNodeFlag := plotCommand.String("node", "node_0", "Sets the node to plot")
	plotWindowFlag := plotCommand.Uint("window", 1, "Sets the sliding window of the plot")
	plotOutputFlag := plotCommand.String("output", "output.png", "Sets the output path")
	plotDurationFlag := plotCommand.Uint("duration", 600, "Sets the time span for the plot")

	checkCommand := flag.NewFlagSet("check", flag.ExitOnError)
	checkNodeListFlag := checkCommand.String("nodelist", "nodes.txt", "Sets the path to the node list file")
	checkVerboseFlag := checkCommand.Bool("verbose", false, "Enables verbose mode")

	if len(os.Args) < 2 {
		fmt.Println("Subcommands: log, plot, check")
		os.Exit(1)
	}

	switch os.Args[1] {
	case "log":
		logCommand.Parse(os.Args[2:])
		log(*intervalFlag, *durationFlag, *nodeListFlag, *dataDirFlag, *numShardsFlag, *targetFileFlag, *grafanaFlag, *displayFlag)
	case "plot":
		plotCommand.Parse(os.Args[2:])
		plot(*plotNodeListFlag, *plotDataDirFlag, *plotContentFlag, *plotNodeFlag, *plotOutputFlag, *plotWindowFlag, *plotDurationFlag)
	case "check":
		checkCommand.Parse(os.Args[2:])
		check(*checkNodeListFlag, *checkVerboseFlag)
	default:
		fmt.Println("Subcommands: log, plot, check")
		os.Exit(1)
	}
}
