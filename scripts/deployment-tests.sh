# Input texts
curl --request POST \
     --url "http://localhost:8080/v1/input" \
     --header "Content-Type: application/json" \
     --data '{
        "texts": ["Hello, World!", "Goodbye, World!"]
     }'


# Fetch similar texts
curl --request POST \
     --url "http://localhost:8080/v1/fetch_similar" \
     --header "Content-Type: application/json" \
     --data '{
        "text": "Goodbye, World!",
        "top_k": 2
     }'