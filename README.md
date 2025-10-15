# BranchSpawn

A CLI tool for creating dedicated database containers for your git branches. Perfect for development workflows where each feature branch needs its own isolated database environment.

## What it does

BranchSpawn automatically creates and manages PostgreSQL containers that are named after your git branches. It uses Docker Compose to spin up database containers, handles port conflicts, and manages container lifecycle based on your current git branch or a specified branch name.

## Installation

### Download from GitHub Releases

Download the appropriate binary for your platform from the [releases page](https://github.com/owen-hope11/branchspawn/releases).

#### Linux
```bash
curl -L https://github.com/owen-hope11/branchspawn/releases/download/<Version_Number>/branchspawn-ubuntu-latest -o branchspawn
chmod +x branchspawn
./branchspawn
```

#### macOS
Due to macOS Gatekeeper restrictions on unsigned binaries, use curl to download and install:

```bash
curl -L https://github.com/owen-hope11/branchspawn/releases/download/<Version_Number>/branchspawn-macos-latest -o branchspawn
chmod +x branchspawn
./branchspawn
```

#### Windows
```powershell
Invoke-WebRequest -Uri https://github.com/owen-hope11/branchspawn/releases/download/<Version_Number>/branchspawn-windows-latest.exe -OutFile branchspawn.exe
.\branchspawn.exe
```

### Add to PATH (Optional)

To use `branchspawn` from anywhere without `./`, add it to your system's PATH:

#### Linux/macOS
```bash
# Move to a directory in your PATH
sudo mv branchspawn /usr/local/bin/

# Or add to user bin directory (create if it doesn't exist)
mkdir -p ~/.local/bin
mv branchspawn ~/.local/bin/
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### Windows
```powershell
# Move to a directory in your PATH (e.g., C:\Program Files\branchspawn)
# Or add current directory to PATH environment variable
```

### Build from Source
```bash
git clone https://github.com/owen-hope11/branchspawn.git
cd branchspawn
cargo build --release
./target/release/branchspawn
```

## Usage

### Prerequisites
- Docker installed and running
- A `compose.yml` file in your project root (or specify path with `-c`)
- Git repository

### Basic Usage

```bash
# Use current git branch to create/start container
branchspawn

# Specify a branch name
branchspawn -b feature/new-login

# Use custom compose file
branchspawn -c ./docker/postgres.yml

# Debug mode
branchspawn -v
```

### Command Line Options

- `-b, --branch-name <NAME>`: Specify branch name (defaults to current git branch)
- `-c, --compose-file-path <PATH>`: Path to compose file (defaults to `compose.yml`)
- `-d, --debug`: Enable debug output (use multiple times for more verbosity)

### How it works

1. **Branch Detection**: Gets current git branch or uses specified branch name
2. **Container Naming**: Creates container name like `feature-new-login-postgres`
3. **Port Conflict Check**: Checks for existing containers using port 5432
4. **Container Management**: Creates new container or starts existing one
5. **Environment Setup**: Sets `CONTAINER_NAME` environment variable for compose file

## Example compose.yml

```yaml
services:
  postgres:
    image: postgres:15
    container_name: ${CONTAINER_NAME}
    environment:
      POSTGRES_PASSWORD: password
      POSTGRES_DB: myapp
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

## License

MIT License - see LICENSE file for details.