#!/bin/bash

echo "Starting to populate the local db"

# Get the current branch name
BRANCH=$(git branch --show-current)
echo "Current branch: $BRANCH"

# Sanitize branch name for Docker container naming (replace '/' with '-')
BRANCH_SANITIZED=${BRANCH//\//-}
CONTAINER_NAME="postgres-${BRANCH_SANITIZED}"
COMPOSE_PROJECT_NAME="curative-${BRANCH_SANITIZED}"
VOLUME_NAME="postgres-data-${BRANCH_SANITIZED}"

# Set environment variables needed by docker-compose
export CONTAINER_NAME
export COMPOSE_PROJECT_NAME
export BRANCH
export BRANCH_SANITIZED

# Check if any container is using port 5432
PORT_CONFLICT=$(docker ps --format "{{.Names}}" --filter "publish=5432" | grep -v "^$CONTAINER_NAME$")
if [ -n "$PORT_CONFLICT" ]; then
    echo "Found other containers using port 5432: $PORT_CONFLICT"
    read -p "Do you want to stop these containers to avoid port conflicts? (y/N): " STOP_OTHERS
    if [[ $STOP_OTHERS == [Yy]* ]]; then
        echo "Stopping containers using port 5432..."
        docker stop $PORT_CONFLICT
    else
        echo "Warning: Port conflict may prevent your database from starting correctly."
    fi
fi

# Check if a container for this branch already exists
if docker container ls -a --filter "name=$CONTAINER_NAME" --format "{{.Names}}" | grep -q "$CONTAINER_NAME"; then
    echo "Container for branch '$BRANCH' already exists."
    
    read -p "Do you want to reinitialize the database? This will delete all existing data. (y/N): " REINIT
    if [[ $REINIT == [Yy]* ]]; then
        echo "Stopping and removing container..."
        docker container stop $CONTAINER_NAME 2>/dev/null || true
        docker container rm $CONTAINER_NAME 2>/dev/null || true
        
        echo "Removing volume..."
        docker volume rm $VOLUME_NAME 2>/dev/null || true
        
        echo "Creating new container for branch '$BRANCH'..."
        docker compose -f $(dirname "$0")/compose.yml up -d
    else
        echo "Starting existing container..."
        docker compose -f $(dirname "$0")/compose.yml start
    fi
else
    echo "Creating new container for branch '$BRANCH'..."
    docker compose -f $(dirname "$0")/compose.yml up -d
fi

# Better debugging
echo "Docker containers currently running:"
docker ps

# Verify container is running
echo "Verifying container is running..."
if docker container ls --filter "name=$CONTAINER_NAME" --format "{{.Names}}" | grep -q "$CONTAINER_NAME"; then
    echo "Container $CONTAINER_NAME is running successfully"
else
    echo "Error: Container $CONTAINER_NAME failed to start"
    echo "Checking container logs:"
    docker logs $CONTAINER_NAME 2>&1 || echo "No logs available"
    exit 1
fi

echo "Database is ready for use with branch '$BRANCH'"

# Improved database initialization verification
echo "Verifying database initialization..."
MAX_RETRIES=10
RETRY_COUNT=0
DB_INITIALIZED=false

while [ $RETRY_COUNT -lt $MAX_RETRIES ] && [ "$DB_INITIALIZED" = false ]; do
    echo "Checking database (attempt $(($RETRY_COUNT + 1))/$MAX_RETRIES)..."
    
    if docker exec $CONTAINER_NAME psql -U postgres -c "\l" 2>/dev/null | grep -q "curative_health_plan"; then
        DB_INITIALIZED=true
        echo "Database initialized successfully!"
    else
        RETRY_COUNT=$((RETRY_COUNT + 1))
        if [ $RETRY_COUNT -lt $MAX_RETRIES ]; then
            echo "Database not ready yet, waiting 3 seconds..."
            sleep 3
        fi
    fi
done

if [ "$DB_INITIALIZED" = false ]; then
    echo "Warning: curative_health_plan database might not have been initialized properly"
    echo "Checking initialization logs:"
    docker logs $CONTAINER_NAME 2>&1 | grep -i "init\|dump\|error\|fail" | tail -20
    
    # Offer to view more detailed logs
    read -p "Do you want to see the full database container logs? (y/N): " VIEW_LOGS
    if [[ $VIEW_LOGS == [Yy]* ]]; then
        docker logs $CONTAINER_NAME
    fi
fi