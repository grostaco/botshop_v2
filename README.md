# BotShop

A passion project for a self-hosted discord bot aimed to keep track of routine and upcoming tasks to be completed with a point system for the user to reward themselves.
# Installation 

## Windows
Download the latest binary file from the releases tab

## *nix systems
Binary releases are not yet available for *nix and macOS systems, it is recommended to build from source by cloning the project and run:
```console
cargo build --release
```

# Setup
The bot requires three environmental variables to be set:
| Variable       | Notes
|----------------|--------------------------------------------------------|
| DISCORD_TOKEN  | The bot token of the account to run this program on    | 
| GUILD_ID       | The guild ID to register the slash commands on         | 
| APPLICATION_ID | The ID of the bot's application to run this program on | 

## Windows
Using Windows' powershell, do:

```console
PS > $env:DISCORD_TOKEN="your token"
PS > $env:GUILD_ID="your guild id"
PS > $env:APPLICATION_ID="your application id" 
```

or set them permanently through your User variables.
## Linux

```console
username@hostname:~$ export DISCORD_TOKEN='your token'
username@hostname:~$ export GUILD_ID='your guild id'
username@hostname:~$ export APPLICATION_ID='your application id'
```

# Acknowledgements

Made with <3 for my girlfriend.