# Data-Plumber

## Introduction
`data-plumber` is a Rust-based tool designed to facilitate the manual synchronization of different databases and data sources. This project serves as both a learning exercise in Rust and a practical solution to compare and manage data across various environments.

### Supported Data Sources
- **MySQL** databases from various URLs (implemented)
- **Neo4j** databases (future implementation)
- Local **JSON** files

## Features
- **Compare Result Sets**: Allows comparison of data across different sources.
- **Generate SQL Inserts**: Produces `.sql` files for database updates.
- **Output JSON Files**: Generates JSON files based on the data queried from sources.

## General Principles
- **Supervision**: There is no direct writing to databases; this tool generates `.sql` files that users can execute manually to ensure data integrity.
- **Memory Management**: Intermediate data is held in memory; the tool is not designed for huge datasets unless sufficient RAM is available.
- **Flexibility**: Configuration of data pipelines is managed via simple JSON configuration files, making it easy to adapt to different data tasks.

## Getting Started

### Prerequisites
- Rust
- Cargo (Rust's package manager)

### Installation
Clone this repository and build the project using Cargo:

```bash
git clone https://github.com/yourusername/data-plumber.git
cd data-plumber
cargo build --release
```

### Configuration

To configure the tool, edit the config.json file in the root directory to match your specific data handling requirements.
Usage

Execute the tool using:
```bash
cargo run --release
```

### Contributing
Contributions are welcome! Please feel free to fork the repository, make your changes, and submit a pull request.

### License
This project is licensed under the MIT License - see the LICENSE.md file for details.

### Acknowledgements
This project is an exercise for learning Rust.
Special thanks to the Rust community for providing extensive documentation and resources.
