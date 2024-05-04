
# Define the website URL
#URL="http://161.97.153.120:12001"
URL="https://www.corgi.wiki"

# Initialize counters
successful_attempts=0
failed_attempts=0

# Loop for a thousand times
for ((i = 1; i <= 100; i++)); do
    echo "Attempt $i:"

    # Perform the curl request verbosely
    curl_output=$(curl -v "$URL" 2>&1)
    echo "$curl_output" > curl_output.txt

    # Check if SSL connection is successful
    if grep -q "SSL connection using" curl_output.txt; then
        echo "SSL connection successful"
        ((successful_attempts++))
    else
        echo "SSL connection failed. Error details:"
        echo "$curl_output"
        ((failed_attempts++))
    fi

    # Print the counts
    echo "Successful attempts: $successful_attempts"
    echo "Failed attempts: $failed_attempts"

    # Clean up temporary file
    rm curl_output.txt

    # Add a delay to not overload the server
    sleep 1
done

# Anfragen an test2.corgijan.dev
#Successful attempts: 36
#Failed attempts: 64


