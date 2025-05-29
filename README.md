# surtr

A Rust Implementation of SURTs based off of the Internet Archive's [Python Implementation](https://github.com/internetarchive/surt).

It aims to be as close to 100% compatiable as possible.

> [!IMPORTANT]
> This is a first draft implementation. THIS IS NOT READY FOR PRODUCTION CODE.


TODO:
    - Build Pipelines to make PySurtr wheels
    - Look into building pipelines to automate the go_surtr library generation.


|              Surtr Option              | Stage Affected   | Default | Description                                                                                               |
| :------------------------------------: | :--------------- | :-----: | :-------------------------------------------------------------------------------------------------------- |
|             public_suffix              | SURT Generation  |  false  | Discard any subdomains in the URL.                                                                        |
|                  surt                  | SURT Generation  |  true   | Return the URL as a SURT. Returns as a valid URL if false.                                                |
|             reverse_ipaddr             | SURT Generation  |  true   | Reverses the IP address in the SURT. Only valid when surt=true                                            |
|              with_scheme               | SURT Generation  |  true   | Includes the scheme (http, dns, ftp) as part of the SURT.                                                 |
|             trailing_comma             | SURT Generation  |  false  | Append a comma after the host portion of the URL.                                                         |
|             host_lowercase             | Canonicalization |  true   | Convert the host portion of the URL into lowercase.                                                       |
|              host_massage              | Canonicalization |  true   | Remove superflous www. from the host. "                                                                   |
|            auth_strip_user             | Canonicalization |  true   | Remove all basic auth from the URL.                                                                       |
|            auth_strip_pass             | Canonicalization |  true   | Remove only the password from basic auth.                                                                 |
|           port_strip_default           | Canonicalization |  true   | Remove the port number if it is the default for the given supported protocol. (http, https are supported) |
|            path_strip_empty            | Canonicalization |  false  | Remove the trailing slash if there is no other path options.                                              |
|             path_lowercase             | Canonicalization |  true   | Convert the path to lowercase.                                                                            |
|         path_strip_session_id          | Canonicalization |  true   | Strip common session ID formats from the path. Supports ASPX.net session IDs.                             |
| path_srtip_trailing_slash_unless_empty | Canonicalization |  true   | Strip the trailing slash from the URL path, unless there is no other path elements.                       |
|         query_strip_session_id         | Canonicalization |  true   | Strip the common session IDs from the query parameters.                                                   |
|            query_lowercase             | Canonicalization |  true   | Convert all elements of the query parameters to lowercase.                                                |
|          query_alpha_reorder           | Canonicalization |  true   | Reorder the query parameters into alphabetical order.                                                     |
|           query_strip_empty            | Canonicalization |  true   | Remove the query parameter ? if there aren't any query parameters.                                        |