**How did you spend your day?**

![plot screenshot](docs/screenshots/2020-12-14-15-15-11.png)

**Track which projects you are working on and how much**

![timeline example](docs/screenshots/2020-12-14-15-15-48.png)

**Add your own custom classification rules...**

![custom rules](docs/screenshots/2020-12-14-15-27-01.png)

**...to get more detail**

![detailed plot screenshot](docs/screenshots/2020-12-14-15-15-20.png)

# Automatic Time Tracker

Track what you spend your time on and stores it in a database. Inspired by [arbtt](https://arbtt.nomeata.de/), which I used previously.

Provides a Web UI to analyze it and create custom rules to improve the classification.

The user activity is tracked as "events", where each events has a timestamp, a duration, and a set of tags with values. You can think of tags as basically an extension of categories, allowing multiple category trees.

For example, an event can may have the following tags:

-   category:Productivity/Software Development/IDE
-   project:2021/timetrackrs
-   device:Home-PC

Which means in the UI you can figure out the time spent on software development, or on a specific project (not necessarily software development), or on a specific device.

## Data Sources

### Working Data Sources

-   Linux X11 tracking. Tracks the following properties:
    -   Which program is open (binary name)
    -   The window title
    -   Which file does the program have open (via cmd args)
    -   Connected WiFi (to be able to figure out rough location)
-   [App Usage](https://play.google.com/store/apps/details?id=com.a0soft.gphone.uninstaller&hl=en) impport

    Allows tracking which apps / app categories are used on your Android devices.

    Adds the following tags:

    -   software-window-title:...
    -   software-executable-path:...
    -   software-window-class:<X11 window class>
    -   software-opened-file:<file path>

-   Browser Usage

    Tracks which websites / domains are used.

    -   For Firefox, install [Add URL to Window Title](https://addons.mozilla.org/en-US/firefox/addon/add-url-to-window-title/) and enable "Show the full URL"
    -   For Chromium-based browsers, install [URL in title](https://chrome.google.com/webstore/detail/url-in-title/ignpacbgnbnkaiooknalneoeladjnfgb?hl=en).

    Adds the following tags:

    -   browse-url:https://...
    -   browse-full-domain:news.ycombinator.com
    -   browse-domain:ycombinator.com

-   VSCode

    Tracks which software development projects you spend your time on, as well as which files.

    To enable, open your user settings and set `window.title` to `${dirty}${activeEditorShort}${separator}${rootName}${separator}${appName}} 🛤sd🠚proj=${rootPath}🙰file=${activeEditorMedium}🠘 `

    Adds the following tags:

    -   software-development-project:<project-path>

-   [Sleep As Android](https://play.google.com/store/apps/details?id=com.urbandroid.sleep&hl=en&gl=US) import

    Imports data of when and how you slept from the Sleep app.

    Creates events with the following tags:

    -   physical-activity:sleeping

-   Timetrackrs import

    Imports data from a different timetrackrs database (e.g. from another device).

-   ZSH shell usage

    To enable, install [zsh-histdb](https://github.com/larkery/zsh-histdb), then add the following to your `.zshrc`:

    ```zsh
    # set window title for timetrackrs
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

        title '' '{"t":"shell","cwd":${(qqq)cwd},"histdb":$HISTDB_SESSION,"usr":${(qqq)user},"cmd":${(qqq)cmd}}'
    }
    add-zsh-hook precmd title_precmd
    add-zsh-hook preexec title_preexec

    ```

### Todo Data Sources

-   Fix Windows data source
-   More detailed browser usage (which containers are used, how did you get to website X?). Needs own webextension
-   mpv usage via (which TV shows and movies watched), via own mpv tracking lua script `.config/mpv/scripts/logall.lua`
-   Google Fitness import via API
-   Manual entry UI to add start/stop times and categories by hand.

## External Info Fetchers

Timetrackrs supports fetching additional information from external sources.

Currently, the following are implemented:

-   Youtube Meta Fetcher

    Fetches some metadata when watching videos like the youtube category (Music / Educational / Entertainment / etc) and the channel.
    Adds the following tags:

    -   youtube-channel:<uploader channel id>
    -   youtube-channel-name:<uploader username>
    -   youtube-tag:<tag-value> for each tag
    -   youtube-category:<category> for each video category

-   Wikidata fetcher

    For each domain visited, tries to get some info about that domain from Wikidata. Adds the following tags.

    Adds the following tags when visiting e.g. reddit.com:

    -   wikidata-label:Reddit
    -   wikidata-id:Q1136
    -   wikidata-category:social networking service
    -   wikidata-category:social-news website
    -   wikidata-category:mobile app

## General Todo

-   Make it easier to setup:

    -   Create a single binary that starts server, api handler and tracking
    -   Create installable systemd service [timetrackrs.service](timetrackrs.service)

-   Look at similar tools, e.g. https://www.raymond.cc/blog/check-application-usage-times-personal-activity-monitor/ , activitywatch, https://www.software.com/code-time

-   Create Android app that uploads the supported app data to the timetrackrs server (currently needs manual file copies)

-   Finish decentralized WebRTC sync support
-   Prettier web frontend

### Ideas for getting program metadata

Metadata we could potentially get:

-   Get open files from /proc/pid/fd
-   This program name can be mapped to a software package using the system package manager, example: `pacman -Qo /usr/bin/vlc`. Then that package name can be used to get metadata, for example the software homepage, tags etc.

## Technical details

### Philosophy

Store as much information in an as raw as possible format in the capture step. Interpret / make it usable later in the analyse step. This prevents accidentally missing interesting information when saving and can allow reinterpretions in unexpected ways later. Redundancies in the data which cause large storage requirements will be solved with compression later.

This is similar to arbtt, and specifically different to many other time tracking alternatives such as ActivityWatch, which stores processed data only.

### Compression notes

Finish and make use of https://github.com/phiresky/sqlite-zstd. Then redundancy in the stored raw events should become basically irrelevant.

Compression benchmark:

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
