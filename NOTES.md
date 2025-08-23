## TODO
- Interprocess communication
- Internal infrastructure
    - Message types
    - More general purpose function hooking mechanism
    - Ability to activate things in game
- Archipelago
- Text client?
- Apworld
- Better dll injection (include_bytes! the dll and unpack in an appdata folder)
- data saving
    - save data for game
    - configuration
- content mod (overlay)


## Intended Loader App flow:
- Detect/Select Bugsnax exe location
- Launch Bugsnax and inject our dll (snax_lib)
- establish IPC with snax_lib
- Get archipelago connection info
- Connect to archipelago
  - Send messages from archipelago to snax_lib
  - Send messages from snax_lib to archipelago
  - Optional: Display textclient stuff to user
