#!/bin/bash

# Optimized large JSON generator for 1GB test file
# Uses streaming and efficient batch processing for maximum performance

OUTPUT_FILE="${1:-input/large_dataset.json}"
TARGET_SIZE_GB="${2:-1}"

echo "Generating optimized large JSON file (${TARGET_SIZE_GB}GB)..."

# Create the input directory if it doesn't exist
mkdir -p "$(dirname "$OUTPUT_FILE")"

# Pre-generate random data arrays for faster access
first_names=("John" "Jane" "Bob" "Alice" "Charlie" "Diana" "Eve" "Frank" "Grace" "Henry" "Oliver" "Emma" "Liam" "Sophia" "Noah" "Isabella" "William" "Charlotte" "James" "Amelia" "Benjamin" "Mia" "Lucas" "Harper" "Mason" "Evelyn" "Ethan" "Abigail" "Alexander" "Emily" "Michael" "Elizabeth" "Sebastian" "Victoria" "Daniel" "Aria" "Matthew" "Scarlett" "Jackson" "Chloe")
last_names=("Smith" "Johnson" "Williams" "Brown" "Jones" "Garcia" "Miller" "Davis" "Rodriguez" "Martinez" "Wilson" "Anderson" "Taylor" "Thomas" "Hernandez" "Moore" "Martin" "Jackson" "Thompson" "White" "Lopez" "Lee" "Gonzalez" "Harris" "Clark" "Lewis" "Robinson" "Walker" "Perez" "Hall" "Young" "Allen" "King" "Wright" "Scott" "Torres" "Nguyen" "Hill" "Flores" "Green")
cities=("New York" "Los Angeles" "Chicago" "Houston" "Phoenix" "Philadelphia" "San Antonio" "San Diego" "Dallas" "San Jose" "Austin" "Jacksonville" "Fort Worth" "Columbus" "Charlotte" "San Francisco" "Indianapolis" "Seattle" "Denver" "Washington" "Boston" "Nashville" "Baltimore" "Louisville" "Portland" "Oklahoma City" "Milwaukee" "Las Vegas" "Albuquerque" "Tucson" "Fresno" "Sacramento" "Kansas City" "Mesa" "Virginia Beach" "Atlanta" "Colorado Springs" "Omaha" "Raleigh" "Miami")
departments=("Engineering" "Marketing" "Sales" "HR" "Finance" "Operations" "Legal" "IT" "Research" "Customer Service" "Product" "Design" "Quality Assurance" "Business Development" "Administration" "Accounting" "Security" "Training" "Procurement" "Analytics" "DevOps" "Support" "Consulting" "Strategy" "Communications" "Compliance" "Risk Management" "Project Management" "Data Science" "UX/UI")
all_skills=("Python" "JavaScript" "Java" "C++" "SQL" "React" "Node.js" "Docker" "Kubernetes" "AWS" "Git" "Linux" "Excel" "Tableau" "Machine Learning" "TypeScript" "Go" "Rust" "MongoDB" "PostgreSQL" "Redis" "GraphQL" "Vue.js" "Angular" "Swift" "Kotlin" "C#" "PHP" "Ruby" "Scala" "TensorFlow" "PyTorch" "Spark" "Hadoop" "Jenkins" "Terraform" "Ansible" "Prometheus" "Grafana" "Elasticsearch")

# Pre-calculate array lengths
first_names_len=${#first_names[@]}
last_names_len=${#last_names[@]}
cities_len=${#cities_len[@]}
departments_len=${#departments[@]}
skills_len=${#all_skills[@]}

# Target size in bytes
target_size_bytes=$((TARGET_SIZE_GB * 1024 * 1024 * 1024))

# Function to generate skills array efficiently using bash arrays
generate_skills_fast() {
    local num_skills=$((2 + RANDOM % 4))  # 2-5 skills
    local skills="["
    local selected=""

    for j in $(seq 1 $num_skills); do
        local skill_index=$((RANDOM % skills_len))
        local skill=${all_skills[$skill_index]}

        # Simple duplicate avoidance
        if [[ ! "$selected" =~ "$skill" ]]; then
            selected="$selected $skill"
            if [ $j -gt 1 ]; then
                skills="$skills, "
            fi
            skills="$skills\"$skill\""
        else
            # Try next skill if duplicate
            num_skills=$((num_skills + 1))
        fi
    done
    skills="$skills]"
    echo "$skills"
}

# Function to generate a user JSON block efficiently
generate_user() {
    local user_id=$1
    local age=$((22 + RANDOM % 58))
    local balance_int=$((RANDOM % 50000))
    local balance_dec=$((RANDOM % 100))
    local balance="$balance_int.$(printf "%02d" $balance_dec)"

    local first_name=${first_names[$((RANDOM % first_names_len))]}
    local last_name=${last_names[$((RANDOM % last_names_len))]}
    local city=${cities[$((RANDOM % cities_len))]}
    local department=${departments[$((RANDOM % departments_len))]}
    local salary=$((35000 + RANDOM % 150000))

    local email="${first_name,,}.${last_name,,}@company.com"
    local skills=$(generate_skills_fast)

    local active=$([ $((RANDOM % 4)) -ne 0 ] && echo "true" || echo "false")
    local newsletter=$([ $((RANDOM % 3)) -ne 0 ] && echo "true" || echo "false")
    local verified=$([ $((RANDOM % 5)) -ne 0 ] && echo "true" || echo "false")

    # Generate project history
    local projects_count=$((RANDOM % 8))
    local projects="["
    for p in $(seq 1 $projects_count); do
        if [ $p -gt 1 ]; then
            projects="$projects, "
        fi
        projects="$projects{\"name\": \"Project$((RANDOM % 1000))\", \"role\": \"Developer\", \"duration_months\": $((1 + RANDOM % 24))}"
    done
    projects="$projects]"

    printf '    {
      "id": %d,
      "name": {
        "first": "%s",
        "last": "%s"
      },
      "email": "%s",
      "age": %d,
      "balance": %s,
      "address": {
        "city": "%s",
        "state": "ST",
        "country": "USA",
        "zip": "%05d"
      },
      "employment": {
        "department": "%s",
        "position": "Senior %s",
        "salary": %d,
        "start_date": "2020-%02d-%02d",
        "manager_id": %d
      },
      "skills": %s,
      "projects": %s,
      "preferences": {
        "newsletter": %s,
        "notifications": %s,
        "theme": "%s"
      },
      "metadata": {
        "created": "2024-%02d-%02dT%02d:%02d:%02dZ",
        "last_login": "2025-%02d-%02dT%02d:%02d:%02dZ",
        "active": %s,
        "verified": %s,
        "login_count": %d
      }
    }' \
    "$user_id" "$first_name" "$last_name" "$email" "$age" "$balance" \
    "$city" $((10000 + RANDOM % 89999)) \
    "$department" "$department" "$salary" \
    $((1 + RANDOM % 12)) $((1 + RANDOM % 28)) $((1000 + RANDOM % 8999)) \
    "$skills" "$projects" \
    "$newsletter" "$verified" \
    "$([ $((RANDOM % 2)) -eq 0 ] && echo "dark" || echo "light")" \
    $((1 + RANDOM % 12)) $((1 + RANDOM % 28)) $((RANDOM % 24)) $((RANDOM % 60)) $((RANDOM % 60)) \
    $((1 + RANDOM % 12)) $((1 + RANDOM % 28)) $((RANDOM % 24)) $((RANDOM % 60)) $((RANDOM % 60)) \
    "$active" "$verified" $((RANDOM % 1000))
}

# Start JSON with metadata
cat > "$OUTPUT_FILE" << EOF
{
  "metadata": {
    "version": "2.0",
    "timestamp": "$(date -Iseconds)",
    "target_size_gb": $TARGET_SIZE_GB,
    "data_source": "optimized_large_synthetic_generator",
    "schema_version": "1.2"
  },
  "users": [
EOF

echo "Starting optimized large file generation..."
echo "Target size: ${TARGET_SIZE_GB}GB (${target_size_bytes} bytes)"

user_count=0
batch_size=500  # Larger batches for better performance
current_size=0

# Status tracking
start_time=$(date +%s)
last_report_time=$start_time

while [ $current_size -lt $target_size_bytes ]; do
    # Generate a batch of users
    batch_content=""
    for i in $(seq 1 $batch_size); do
        user_count=$((user_count + 1))

        user_json=$(generate_user $user_count)
        batch_content="$batch_content$user_json"

        # Add comma except for potential last user
        if [ $current_size -lt $((target_size_bytes - 50000)) ]; then  # Leave some buffer
            batch_content="$batch_content,"
        fi
        batch_content="$batch_content\n"
    done

    # Write batch to file
    printf "$batch_content" >> "$OUTPUT_FILE"

    # Check file size every few batches for efficiency
    if [ $((user_count % (batch_size * 5))) -eq 0 ]; then
        current_size=$(stat -c%s "$OUTPUT_FILE" 2>/dev/null || stat -f%z "$OUTPUT_FILE" 2>/dev/null || echo 0)
        current_mb=$((current_size / 1024 / 1024))

        # Progress reporting
        current_time=$(date +%s)
        if [ $((current_time - last_report_time)) -ge 10 ]; then  # Report every 10 seconds
            elapsed=$((current_time - start_time))
            rate=$((user_count / (elapsed + 1)))
            eta_seconds=$(((target_size_bytes - current_size) / (current_size / (elapsed + 1))))

            printf "\rProgress: %d users, %dMB/%dMB (%.1f%%), Rate: %d users/sec, ETA: %dm%ds" \
                "$user_count" "$current_mb" "$((target_size_bytes / 1024 / 1024))" \
                "$(echo "scale=1; $current_size * 100 / $target_size_bytes" | bc -l 2>/dev/null || echo "0")" \
                "$rate" "$((eta_seconds / 60))" "$((eta_seconds % 60))"

            last_report_time=$current_time
        fi
    fi
done

# Close JSON structure
cat >> "$OUTPUT_FILE" << EOF
  ],
  "summary": {
    "total_users": $user_count,
    "generation_completed_at": "$(date -Iseconds)",
    "generation_time_seconds": $(($(date +%s) - start_time))
  }
}
EOF

final_size=$(stat -c%s "$OUTPUT_FILE" 2>/dev/null || stat -f%z "$OUTPUT_FILE" 2>/dev/null || echo 0)
final_mb=$((final_size / 1024 / 1024))
total_time=$(($(date +%s) - start_time))

echo
echo "✓ Generated $OUTPUT_FILE with $user_count users"
echo "✓ Final file size: ${final_mb}MB ($(echo "scale=2; $final_size / 1024 / 1024 / 1024" | bc -l)GB)"
echo "✓ Generation time: ${total_time} seconds"
echo "✓ Average rate: $((user_count / (total_time + 1))) users/second"

# Validate JSON syntax if jq is available
if command -v jq >/dev/null 2>&1; then
    echo "Validating JSON syntax..."
    if timeout 30 jq empty "$OUTPUT_FILE" >/dev/null 2>&1; then
        echo "✓ JSON syntax is valid"
    else
        echo "✗ JSON syntax validation timed out or failed (file may be too large for jq)"
    fi
else
    echo "Note: jq not available for JSON validation"
fi
