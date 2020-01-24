# track pc usage

track which programs is used how much and stores data in database. Inspired by [arbtt](https://arbtt.nomeata.de/), which I used previously.

## todo

-   analyse data (ui / graphs etc)
-   autoimport more detailed / other data
    -   (phone usage via [App Usage](https://play.google.com/store/apps/details?id=com.a0soft.gphone.uninstaller&hl=en))
    -   browser usage via own firefox/chrome `permanent-history-webextension`, tbd
    -   mpv usage via own mpv tracking lua script `.config/mpv/scripts/logall.lua` tbu
    -   shell usage via zsh-histdb
-   make non-crap
-   look at similar tools, e.g. https://www.raymond.cc/blog/check-application-usage-times-personal-activity-monitor/

## philosophy

Store as much information in an as raw as possible format in the capture step. Interpret / make it usable later in the analyse step. This prevents accidentally missing interesting information when saving and can allow reinterpretions in unexpected ways later. Redundancies in the data which cause large storage requirements will be solved with compression later.

## todo:

remove Defaults from deserializing in x11.rs

## notes

db rows:

-   timestamp
-   sampling method used
-   data

time sampling. decide between random sampling, stratified sampling or grid (?) sampling

## Data Sources Setup

### Firefox

Install https://addons.mozilla.org/en-US/firefox/addon/add-url-to-window-title/ and enable "Show the full URL"

### VS Code

Open your user settings and set `window.title` to `${dirty}${activeEditorShort}${separator}${rootName}${separator}🛤sd🠚proj=${rootPath}🙰file=${activeEditorMedium}🠘 VSCode`

### Shell / Zsh

Todo: look at https://arbtt.nomeata.de/doc/users_guide/effective-use.html

1. Add / Install [zsh-histdb](https://github.com/larkery/zsh-histdb)

2. Add the following to your zshrc:

    ```zsh
    # set window title for track-pc-usage-rs
    # adopted from https://github.com/ohmyzsh/ohmyzsh/blob/master/lib/termsupport.zsh
    autoload -Uz add-zsh-hook

    function title_precmd {
        title_preexec '' ''
    }
    function title_preexec {
        # http://zsh.sourceforge.net/Doc/Release/Expansion.html
        # http://zsh.sourceforge.net/Doc/Release/Prompt-Expansion.html#Prompt-Expansion
        local cwd="$(print -P '%~')"
        local user="$(print -P '%n@%m')"
        local LINE="$2"
        local cmd="$(print -P '%100>...>$LINE%<<')"

        title '' '{"cwd":${(qqq)cwd},"histdb":$HISTDB_SESSION,"usr":${(qqq)user},"cmd":${(qqq)cmd}}'
    }
    add-zsh-hook precmd title_precmd
    add-zsh-hook preexec title_preexec

    ```

## Compression notes

Think about row-level compression.

```
for id in $(sqlite3 activity.sqlite3 "select id from events where data_type='x11'"); do sqlite3 activity.sqlite3 "select data from events where id='$id'" > "x11/$id.json"; done
```

Zstd test: 7400 x11 events rows:

-   202M uncompressed (27kB avg)
-   21M compressed without dictionary (2.8kbyte avg)
-   20M compressed with `xz -9`
-   5.0M compressed with generated dictionary (675byte avg), 110KB dictionary-file (which is default --maxdict)
-   12M compressed with random sample json file as dictionary (1.6kbyte avg)
-   11M compressed with dict generated 100 random json files (20kb dict file)
-   2.7M compressed with large dict, 1MB dict file size (--maxdict=1000000)
-   1.9MB as single file: `zstd -19 all`
-   1.6MB as single file: `zstd --ultra -22 --long=31 all`
-   1.3MB as single file (ordered by date) `-19`
-   1.3MB as single file (ordered by date) `--ultra -22 --long=31`

Conclusion: zstd is awesome
