## files/base directory
There are files reserved for the BaseOS base and its `util` utility. Feel free to change contets, but make sure you know what are you doing.


If you want to create your own subcommands, put them into `files/base/scripts`
- Make sure to use all the functions that other scripts use too!
  - add `source $PWD/files/base/scripts.conf` at the start of your subcommand