# het

**H**ealth **e**xpense **t**racker through the CLI.

## Why

I track all my health expenses in a single directory in cloud storage.
Each expense follows a naming routine to capture the name, date, and monetary amount.

`het`

- standardizes the process of adding new expenses,
- makes retrieving expenses based on a monetary target easy.

## How

`het` uses [SQLite](https://sqlite.org/) to store expenses and their metadata including:

- name
- date of entry
- file type
- monetary amount
- file content (compressed)

When it's time to pull expenses out, provide `het` a target amount and it will find the combination of unretrieved expenses whose sum is closest to the target.
If there's no such combination, e.g., no amount of expenses add up to the target, `het` will report it to you.

## Usage

To avoid outdated documentation, this README only includes high level details on sub-commands:

- `het add`: add a new expense
- `het retrieve`: provide a target amount and `het` will retrieve and write the best combination of expenses to the target directory

## AI

Parts of this codebase are written with AI.
I review every line and only keep code I would've written myself.
This is a personal project.
There are no deadlines and I will take as long as needed to maintain a high bar in this codebase.
