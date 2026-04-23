for i in $(seq 0 51); do
	IFS= read -r line
    echo "$i,$line" >> $1.fixed
done < $1