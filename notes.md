#### Todo

- Add loading spinner for entry submit button and sort saving
- Implement a sorting endpoint for faster sorting
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
- Add mark as read button
- Make entities editable
- ~~submit should be disabled as long no new entry is configured~~

- implement last_modified for spotify_config and system_config and pull changes. Emit a config changed event once config changed
- Replace psplash with more flexible solution to use same png files as the initramfs does. Maybe add fbv to the image as well?
- make splash rotatable using -a flag on psplash service
- create a setup script which ensures the default state of the system config | or adjust the system config to follow the default state
- what happens when using spotify search without configuring spotify?
    - spotify option should be disabled as long as there is no access token
- implement theming support
- when an error occurs the event handler in user_interface seem to be stuck. Actions are dispatched but nothing is handled. Example was playing a local file
  without file content leading to an error and clicking again on a file did nothing
- Make boot of image fast and stable
- Create an updater to pull updates from github
- Fix admin_interface_server commands as not anymore on dietpi
- reload user_interface on media library change
- Sometimes the admin interface does not allow focussing on the input field
- Going to prev track at the beginning of an album the play status is reset but the tack still plays. I think the prev button should be disabled if there is no
  prev track. Same for the next track
- When searching with spotify for tracks, being in an album of tracks, I can't add new tracks as I have to expand the entry

1. download archive
2. extract archive tar -xf tinyghettobox-raspberrypi3-64.rootfs-20250420220723.update.tar.gz -C /srv/update
3. rm -rf tinyghettobox-raspberrypi3-64.rootfs-20250420220723.update.tar.gz
4. new_partition_no=$(test $current_rootfs_partition == 'a' && echo '3' || echo '2')
4. fs_size=$(stat -c %s rootfs.squashfs)
5. userdata_size=$(parted -s /dev/mmcblk0 unit b print | awk "/ 4 / {print \$4}" | sed 's/[sB]//')
6. new_userdata_size=$((fs_size + userdata_size))
6. resize2fs -s /dev/mmcblk0p4 $new_userdata_size
7. parted -s /dev/mmcblk0 unit b resizepart 4 $new_userdata_size
8. parted -s /dev/mmcblk0 unit b resizepart $new_partition_no $fs_size
9. dd if=/srv/update/rootfs.squashfs of=/dev/mmcblk0p$new_partition_no bs=1M status=progress
10. 