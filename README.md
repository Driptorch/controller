# driptorch-controller
Controller for the Driptorch DNS + Proxy.   
For more information, visit https://driptorch.net/

| [Code](https://git.sr.ht/~eviee/driptorch-controller) | [Issues](https://todo.sr.ht/~eviee/Driptorch) | [Development Mailing List](https://lists.sr.ht/~eviee/driptorch-devel) |
|-------------------------------------------------------|-----------------------------------------------|------------------------------------------------------------------------|

---

## Requirements:
#### Deployment
* PostgreSQL
* RabbitMQ

#### Development
* Rust 1.60+
* PostgreSQL
* RabbitMQ

## Environment Variables:
| **Variable** |                                             **Description**                                              | **Required?** |
|:------------:|:--------------------------------------------------------------------------------------------------------:|:-------------:|
| DATABASE_URL |                                    PostgreSQL database connection URL                                    |       Y       |
|  AMQP_ADDR   |                                 Message queue (RabbitMQ) connection URL                                  |       Y       |
| UAP_REGEXES  | Path to the [BrowserScope UA regex YAML](https://github.com/ua-parser/uap-core/blob/master/regexes.yaml) |       N       |
|   RSA_KEY    |                Path to the RSA private key used to create certificates !!! KEEP THIS SAFE                |       Y       |
|  XCC20_KEY   |            Path to the XChaCha20-Poly1305 key used to encrypt private keys !!! KEEP THIS SAFE            |       Y       |

---

### See also
* [driptorch-client](https://git.sr.ht/~eviee/driptorch-client)
* [driptorch-panel](https://git.sr.ht/~eviee/driptorch-panel)