import csv
import math
import statistics as stat
import matplotlib.pyplot as plt


def main():
	run_time_plot()
	cache_perf_plot()


def run_time_plot():
	throughputs = []
	deviations = []
	counts = [10000,100000,1000000,10000000]
	for x in counts:
		file = f"run_time_benchmark_{x}.csv"
		data = read_run_time_data(file)
		avg_run_time = stat.mean(data)
		dev_run_time = stat.stdev(data)
		throughputs.append(x/avg_run_time)
		deviations.append(dev_run_time)
	

	plt.errorbar(counts, throughputs, yerr=deviations, fmt='o-', capsize=4)

	plt.xscale('log')

	plt.xlabel('Number of Events (events)')
	plt.ylabel('Average Throughput (1e6 events/sec)')
	plt.title('Average Event Throughput vs. Number of Events Generated')

	plt.savefig("throughput_vs_count.png")
	plt.close()


def cache_perf_plot():
	cache_miss_rates = []
	deviations = []
	counts = [10000,100000,1000000,10000000]
	for x in counts:
		file = f"cache_benchmark_{x}.csv"
		data = read_cache_perf_data(file)
		cache_miss_rates.append(stat.mean(data))
		deviations.append(stat.stdev(data))

	plt.errorbar(counts, cache_miss_rates, yerr=deviations, fmt='o-', capsize=4)

	plt.xscale('log')

	plt.xlabel('Number of Events (events)')
	plt.ylabel('Cache Miss Rate (% Cache References)')
	plt.title('Cache Miss Rate vs. Number of Events Generated')

	plt.savefig("cache_perf_vs_count.png")

	
def read_run_time_data(file_name):
	run_times = []
	with open(file_name, 'r') as f:
		reader = csv.DictReader(f)
		for row in reader:
			run_times.append(float(row["run_time"])/1e9)
	return run_times


def read_cache_perf_data(file_name):
	miss_rates = []
	with open(file_name, 'r') as f:
		reader = csv.DictReader(f)
		for row in reader:
			miss_rates.append(100.0*float(row["cache_misses"])/float(row["cache_references"]))
	return miss_rates
	
	

if __name__ == "__main__":
	main()