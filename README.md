# track pc usage

track which programs is used how much and stores data in database. Inspired by [arbtt](https://arbtt.nomeata.de/), which I used previously.

## todo

- analyse data (ui / graphs etc)
- autoimport more detailed / other data
    - (phone usage via [App Usage](https://play.google.com/store/apps/details?id=com.a0soft.gphone.uninstaller&hl=en)
    - browser usage via own firefox/chrome `permanent-history-webextension`, tbd
    - mpv usage via own mpv tracking lua script `.config/mpv/scripts/logall.lua` tbu
- make non-crap
- look at similar tools, e.g. https://www.raymond.cc/blog/check-application-usage-times-personal-activity-monitor/

## philosophy

Store as much information in an as raw as possible format in the capture step. Interpret / make it usable later in the analyse step. This prevents accidentally missing interesting information when saving and can allow reinterpretions in unexpected ways later. Redundancies in the data which cause large storage requirements will be solved with compression later.

## notes

db rows:

- timestamp
- sampling method used
- data

time sampling. decide between random sampling, stratified sampling or grid (?) sampling