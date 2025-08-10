#!/bin/bash

# Optimized JSON generator for test data
# Much faster than the original using efficient bash techniques

OUTPUT_FILE="${1:-input/test_dataset.json}"
NUM_USERS="${2:-1000}"

echo "Generating optimized test JSON file with $NUM_USERS users..."

# Create the input directory if it doesn't exist
mkdir -p "$(dirname "$OUTPUT_FILE")"

# Pre-generate random data arrays for faster access
first_names=("John" "Jane" "Bob" "Alice" "Charlie" "Diana" "Eve" "Frank" "Grace" "Henry" "Oliver" "Emma" "Liam" "Sophia" "Noah" "Isabella" "William" "Charlotte" "James" "Amelia" "Benjamin" "Mia" "Lucas" "Harper" "Mason" "Evelyn" "Ethan" "Abigail" "Alexander" "Emily")
last_names=("Smith" "Johnson" "Williams" "Brown" "Jones" "Garcia" "Miller" "Davis" "Rodriguez" "Martinez" "Wilson" "Anderson" "Taylor" "Thomas" "Hernandez" "Moore" "Martin" "Jackson" "Thompson" "White" "Lopez" "Lee" "Gonzalez" "Harris" "Clark" "Lewis" "Robinson" "Walker" "Perez" "Hall")
cities=("New York" "Los Angeles" "Chicago" "Houston" "Phoenix" "Philadelphia" "San Antonio" "San Diego" "Dallas" "San Jose" "Austin" "Jacksonville" "Fort Worth" "Columbus" "Charlotte" "San Francisco" "Indianapolis" "Seattle" "Denver" "Washington" "Boston" "Nashville" "Baltimore" "Louisville" "Portland" "Oklahoma City" "Milwaukee" "Las Vegas" "Albuquerque" "Tucson")
departments=("Engineering" "Marketing" "Sales" "HR" "Finance" "Operations" "Legal" "IT" "Research" "Customer Service" "Product" "Design" "Quality Assurance" "Business Development" "Administration" "Accounting" "Security" "Training" "Procurement" "Analytics")
all_skills=("Python" "JavaScript" "Java" "C++" "SQL" "React" "Node.js" "Docker" "Kubernetes" "AWS" "Git" "Linux" "Excel" "Tableau" "Machine Learning" "TypeScript" "Go" "Rust" "MongoDB" "PostgreSQL" "Redis" "GraphQL" "Vue.js" "Angular" "Swift" "Kotlin" "C#" "PHP" "Ruby" "Scala")

# Pre-calculate array lengths for efficiency
first_names_len=${#first_names[@]}
last_names_len=${#last_names[@]}
cities_len=${#cities[@]}
departments_len=${#departments[@]}
skills_len=${#all_skills[@]}

# Function to generate random email efficiently
generate_email() {
    local first=$1
    local last=$2
    echo "${first,,}.${last,,}@company.com"
}

# Function to generate skills array efficiently
generate_skills() {
    local num_skills=$((1 + RANDOM % 5))
    local skills="["
    local used_indices=""

    for j in $(seq 1 $num_skills); do
        # Avoid duplicate skills by tracking used indices
        local skill_index
        while true; do
            skill_index=$((RANDOM % skills_len))
            if [[ ! "$used_indices" =~ " $skill_index " ]]; then
                used_indices="$used_indices $skill_index "
                break
            fi
        done

        local skill=${all_skills[$skill_index]}
        if [ $j -gt 1 ]; then
            skills="$skills, "
        fi
        skills="$skills\"$skill\""
    done
    skills="$skills]"
    echo "$skills"
}

# Use a more efficient approach: generate all data in memory first, then write
echo "Pre-generating user data..."

# Start building JSON content
json_content='{
  "metadata": {
    "version": "1.0",
    "timestamp": "'$(date -Iseconds)'",
    "total_users": '$NUM_USERS',
    "data_source": "optimized_synthetic_generator"
  },
  "users": ['

# Generate users in batches for better memory management
batch_size=100
batches=$((NUM_USERS / batch_size))
remainder=$((NUM_USERS % batch_size))

for batch in $(seq 0 $batches); do
    if [ $batch -eq $batches ]; then
        current_batch_size=$remainder
        if [ $remainder -eq 0 ]; then
            break
        fi
    else
        current_batch_size=$batch_size
    fi

    batch_content=""
    start_id=$((batch * batch_size + 1))

    for i in $(seq $start_id $((start_id + current_batch_size - 1))); do
        # Use faster arithmetic operations
        user_id=$i
        age=$((20 + RANDOM % 60))
        balance_int=$((RANDOM % 10000))
        balance_dec=$((RANDOM % 100))
        balance="$balance_int.$(printf "%02d" $balance_dec)"

        # Fast random selections using modulo
        first_name=${first_names[$((RANDOM % first_names_len))]}
        last_name=${last_names[$((RANDOM % last_names_len))]}
        city=${cities[$((RANDOM % cities_len))]}
        department=${departments[$((RANDOM % departments_len))]}
        salary=$((30000 + RANDOM % 120000))

        # Generate email efficiently
        email=$(generate_email "$first_name" "$last_name")

        # Generate skills array
        skills=$(generate_skills)

        # Generate boolean values efficiently
        active=$([ $((RANDOM % 2)) -eq 0 ] && echo "true" || echo "false")
        newsletter=$([ $((RANDOM % 3)) -ne 0 ] && echo "true" || echo "false")

        # Build user JSON efficiently
        user_json='    {
      "id": '$user_id',
      "name": {
        "first": "'$first_name'",
        "last": "'$last_name'"
      },
      "email": "'$email'",
      "age": '$age',
      "balance": '$balance',
      "address": {
        "city": "'$city'",
        "country": "USA"
      },
      "employment": {
        "department": "'$department'",
        "salary": '$salary'
      },
      "skills": '$skills',
      "preferences": {
        "newsletter": '$newsletter'
      },
      "metadata": {
        "created": "'$(date -Iseconds)'",
        "active": '$active'
      }
    }'

        batch_content="$batch_content$user_json"

        # Add comma except for very last user
        if [ $i -lt $NUM_USERS ]; then
            batch_content="$batch_content,"
        fi
        batch_content="$batch_content\n"
    done

    json_content="$json_content$batch_content"
    echo "Generated batch $((batch + 1))/$((batches + 1)) (users $start_id to $((start_id + current_batch_size - 1)))"
done

json_content="$json_content  ]
}"

# Write everything at once for maximum efficiency
echo "Writing JSON file..."
printf "$json_content" > "$OUTPUT_FILE"

echo "Generated $OUTPUT_FILE with $NUM_USERS users"
echo "File size: $(du -h "$OUTPUT_FILE" | cut -f1)"

# Validate JSON syntax
if command -v jq >/dev/null 2>&1; then
    echo "Validating JSON syntax..."
    if jq empty "$OUTPUT_FILE" >/dev/null 2>&1; then
        echo "✓ JSON syntax is valid"
    else
        echo "✗ JSON syntax error detected"
        exit 1
    fi
else
    echo "Note: jq not available for JSON validation"
fi
