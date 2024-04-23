# Data-Plumber

## Introduction
`data-plumber` is a Rust-based tool designed to facilitate the manual synchronization of different databases and data sources. This project serves as both a learning exercise in Rust and a practical solution to compare and manage data across various environments.

## Features
- **Compare Result Sets**: Allows comparison of data across different sources.
- **Generate SQL Inserts**: Produces `.sql` files for database updates.
- **Output JSON Files**: Generates JSON files based on the data queried from sources.

## General Principles
- **Supervision**: There is no direct writing to databases; this tool generates `.sql` files that users can execute manually to ensure data integrity.
- **Memory Management**: Intermediate data is held in memory; the tool is not designed for huge datasets unless sufficient RAM is available.
- **Flexibility**: Configuration of data pipelines is managed via simple JSON configuration files, making it easy to adapt to different data tasks.
- **Restartability**: Some operations can be both expensive and slow, like prompting an LLM for every record in a table. The application should be restartable without wasting the work already done.

### Implementation
Every configured 'Process' is run in the listed order (no automatic dependency resolution yet) sequentially (for now). The input nodes will create a 
table in memory with the same name of the process, so you can use that table as input for subsequent processes.

### Use cases
My basic needs are:
- copy data between many databases, for example from dev to demo, or from Neo4j queries to mysql tables
- compare tables in different databases
- backup a table data before doing some edits
- integrate a table with requests to an LLM

### Roadmap
- ~~JSON input/output~~
- ~~MYSQL input/output~~
- ~~Compare tables~~
- Neo4j input/output
- REST API input/output
- CSV input/output
- JSON processing with jq-like syntax
- Merging/composing tables (inner join, left join)
- Data validation

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
To configure the tool, edit a config.json file, possibly in a new directory, with your pipeline configuration.
Usage

Execute the tool using:
```bash
<$PATH>/data-plumber config.json
```

### Contributing
Contributions are welcome! Please feel free to fork the repository, make your changes, and submit a pull request.

### License
This project is licensed under the MIT License - see the LICENSE.md file for details.

### Acknowledgements
This project started an exercise for learning Rust.
Thanks to [WHP](whp.ai) for sponsoring part of this software.
Thanks to the Rust community for providing extensive documentation and resources.
