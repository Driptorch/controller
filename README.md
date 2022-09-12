# driptorch-controller
Controller for the Driptorch DNS + Proxy.   
For more information, visit https://driptorch.net/

---

## Requirements:
#### Deployment
* PostgreSQL

#### Development
* Rust 1.60+
* PostgreSQL

## Environment Variables:
| **Variable** | **Description**                                                                                          |     | **Required?** |
|--------------|----------------------------------------------------------------------------------------------------------|-----|---------------|
| DATABASE_URL | PostgreSQL database connection URL                                                                       |     | Y             |
| UAP_REGEXES  | Path to the [BrowserScope UA regex YAML](https://github.com/ua-parser/uap-core/blob/master/regexes.yaml) |     | N             |

---

### See also
* [driptorch-client](https://git.sr.ht/~eviee/driptorch-client)
* [driptorch-panel](https://git.sr.ht/~eviee/driptorch-panel)