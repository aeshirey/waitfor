# waitfor
`waitfor` is a shell app for delaying on conditions. It will simply block until any one of the specified conditions is met. (To require multiple conditions, chain `waitfor` calls together with `&&`.)

## Condition types 
| Condition type | Flag | Short | Example | Notes |
|----------------|------|-------|---------|-------|
| Time delay     | `--elapsed spec` | `-t` | `waitfor --elapsed 10m30s`    | Supports only full integer values for `d`, `h`, `m`, and `s` |
| File existence | `--exists filename` | `-e` | `waitfor --exists foo.txt` | Waits until the file `foo.txt` exists |
| HTTP GET       | `--get [code,]url` | `-g` | `--get http://google.com` | Waits until a GET to Google returns [HTTP status code](https://en.wikipedia.org/wiki/List_of_HTTP_status_codes) 200 |
|                |                    | | `--get 404,http://example.com` | Waits until a GET to example.com returns 404 |
| TCP connection | `--tcp host:port` | `-p` | `--tcp 192.168.0.123:22` | Waits until a TCP connection can be made to the specified host. The numeric port must be specified. |
| File modification date | `--update` | `-u` | `waitfor --update log.txt` | Waits until the modification date on `log.txt` changes. If the modification date can't be retrieved, this condition is ignored |
| File size | `--size` | `-s` | `waitfor --size log.txt` | Waits until size of `log.txt` changes. The file must exist initially to be included, but if it ceases to exist, the condition will be triggered. |

## Negation
You can also negate conditions by prefixing the flag with `not` or by uppercasing the short flag:

| Condition type | Flag | Short | Example | Notes |
|----------------|------|-------|---------|-------|
| Duration not yet met | `--not-elapsed` | `-T` | `waitfor --not-elapsed 10m` | See comments below |
| File non-existence | `--not-exists filename` | `-E` | `waitfor --not-exists bar.txt` | Waits until the file `bar.txt` _no longer_ exists |
| HTTP GET | `--not-get [code,]url` | `-G` | `--not-get http://google.com` | Waits until a GET to Google _doesn't_ return HTTP status code 200 |
|          |                    | | `--get 404,http://example.com` | Waits until a GET to example.com returns anything but 404 |
| TCP connection | `--not-tcp host:port` | `-P` | `--not-tcp 192.168.0.123:22` | Waits until a TCP connection _can not_ be made to the specified host (either because the host itself is down or the port isn't available). |
| Modification date stops changing | `--not-update` | `-U` | `waitfor --not-update download.iso` | Waits until the modification date on `download.iso` stops changing. If the modification date can't be retrieved, this condition is ignored. Two identical sequential values trigger this condition. |
| File size stops changing | `--not-size` | `-S` | `waitfor --not-size download.iso` | Waits until size of `download.iso` stops changing. The file must exist initially to be included, but if it ceases to exist, the condition will be triggered. Two identical sequential values also trigger this condition. |

### Not-Elapsed
The `--not-elapsed` flag, by itself, will immediately resolve, but it is intended to be combined with other flags. For example, to delay until a file is deleted but _only_ if it's in the first minute:

```bash
waitfor -T 1m -E foo.txt
```

If foo.txt isn't deleted in the first minute, then the command will block indefinitely. The underlying library, [`waitforit`](https://github.com/aeshirey/waitforit), allows AND/OR combinations for richer expression.

## Additional flags
| Flag | Description |
|------|-------------|
| `--interval n` | The delay in seconds between checks against the specified condition(s). Default is 2 |
| `--verbose` | Writes some details about status checks to stdout. Without this option, nothing is written. |

## Multiple conditions
You can combine any number of the above conditions (except `--elapsed`, which may be used only once), and as soon as any one of them is met, the program will exit. For example, this command will complete when `foobar.txt` is found _or_ after ten minutes has passed:

    waitfor --elapsed 10m --exists foobar.txt

Multiple existence checks are also easy. To wait for either `foo.txt` or `bar.jpg` to exist:

    waitfor --exists foo.txt --exists bar.jpg

For `--exists`, you can combine these into a single flag:

    waitfor --exists foo.txt,bar.jpg

Nonexistence checks are similar but the inversion. When `tmpfile` no longer exists (has been deleted or renamed), this command will complete:

    waitfor --not-exists tmpfile

The `--get` flag executes an HTTP GET action against the specified URL, which is assumed to be valid. You may optionally specify any [HTTP status code](https://en.wikipedia.org/wiki/List_of_HTTP_status_codes) prefixing the URL to wait for that code to be returned, otherwise 200 is assumed. Multiple separate `--get` flags may be provided, but multiple URLs may not be combined in a single flag. For example:

```bash
# Check for either URL to be available as 200 OK:
waitfor --get http://my-site.com/ --get 200,http://your-site.com

# Check for the URL to return 200 or 300
waitfor --get http://my-site.com/ --get 300,http://my-site.com

# Invalid! URLs must be split into two --get flags:
# waitfor --get http://google.com/,http://microsoft.com
# Better:
waitfor --get http://google.com/ --get http://microsoft.com
```

Because HTTP GET calls incur nontrivial latency, the current implementation counts how long each condition takes to run, and that duration is subtracted from the `--interval`. That remaining time is spent sleeping. If multiple GETs are called such that the delay exceeds the interval, a new loop will immediately be started. Invocations are not currently parallelized.
