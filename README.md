# waitfor
`waitfor` is a shell app for delaying on conditions. It will simply block until any one of the specified conditions is met. (To require multiple conditions, chain `waitfor` calls together with `&&`.)

## Condition types 
| Condition type | Flag | Example | Notes |
|----------------|------|---------|-------|
| Time delay     | `--delay spec` | `waitfor --delay 10m30s`    | Supports only full integer values for `h`, `m`, and `s` |
| File existence | `--exists filename` | `waitfor --exists foo.txt` | Waits until the file `foo.txt` exists |
| File non-existence | `--not-exists filename` | `waitfor --not-exists bar.txt` | Waits until the file `bar.txt` _no longer_ exists |
| HTTP GET | `--get [code,]url` | `--get http://google.com` | Waits until a GET to Google returns [HTTP status code](https://en.wikipedia.org/wiki/List_of_HTTP_status_codes) 200 |
|          |                    | `--get 404,http://example.com` | Waits until a GET to example.com returns 404 |

## Additional flags
| Flag | Description |
|------|-------------|
| `--interval n` | The delay in seconds between checks against the specified condition(s). Default is 2 |
| `--verbose` | Writes some details about status checks to stdout. Without this option, nothing is written. |

## Multiple conditions
You can combine any number of the above conditions (except `--elapsed`, which may be used only once), and as soon as any one of them is met, the program will exit. For example, this command will complete when `foobar.txt` is found _or_ after ten minutes has passed:

    waitfor --delay 10m --exists foobar.txt

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

Because HTTP GET calls incur nontrivial latency, the current implementation counts how long each condition takes to run, and that duration is subtracted from the `--interval`. That remaining time is spent sleeping. If multiple GETs are called such that the delay exceeds the interval, a new loop will immediately be stated. Invocations are not currently parallelized.
