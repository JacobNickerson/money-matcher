file=run_time_benchmark_$1.csv
echo "run,run_time,sim_time" > $file 
for i in $(seq 1 $2);
do 
	line=$(../target/release/engine --seed 67 --benchmark poisson --count $1 | cut -d',' -f2-)
	echo "$i,$line" >> $file
done