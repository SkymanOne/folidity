# Verifiable demo

This is a demo contract written in [folidity](https://github.com/SkymanOne/folidity).
It has some arbitrary bounds that allow you to control a dynamic formal specification of the contract.
Folidity compiler allows you to formally prove that these bounds and contracts are not violated in your code.

## Usage
- `folidity new <project_name>` - initialise a new counter project in the destination directory.
- `folidty check <file_name>.fol` - check the source code for parser, semantic and type errors.
- `folidity compile <file_name>.fol` - compile the source code into Teal binary. Produces approval program: `approval.teal`, and clear program: `clear.teal` 

You can run `--help` on `folidity` command and its subcommand to learn more about the CLI.