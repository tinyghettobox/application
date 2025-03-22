#### Todo

- Check memory usage of user_interface when loading local file
- ~~admin-interface server consumes high amount of memory when uploading files and does not free it~~
- ~~icons are missing on user-interface on device~~
- ~~stream is super loud on device~~
    - amixer -c 0 sset Master 80%
    -
- ~~splash image is not rotated properly~~
- ~~open entry on long press~~

- ~~progress of file upload is only a spinner and doesn't show real progress~~
- stream urls should always have a schema
- blurring a stream and having name and url should automatically press the add button
- submit should be disabled as long no new entry is configured

- what happens when using spotify search without configuring spotify?
    - spotify option should be disabled as long as there is no access token
- when an error occurs the event handler in user_interface seem to be stuck. Actions are dispatched but nothing is handled. Example was playing a local file
  without file content leading to an error and clicking again on a file did nothing
- Make boot of image fast and stable
- Create an updater to pull updates from github
- Fix admin_interface_server commands as not anymore on dietpi
- reload user_interface on media library change