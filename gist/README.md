# GitHub Gist Client

PROCESS
=======
msg_urls: Vec<String> = []
get all gists
if gist.description == agent ID:
    if gist.files.len() > 0:
        for file in gist.files:
            msg_urls.add(gist.file.raw_url)


Extism get_gists COMMAND
```
extism call target/wasm32-unknown-unknown/release/c4.wasm c4 --input '{"action": "receive", "params": {"api_key": "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", "agent_id": "12345"}}'  --allow-host "*" --wasi --log-level info
```

INPUT
```
'{"action": "get_gists", "params": {"api_key": "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", "agent_id": "12345"}'
```

curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer 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" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/gists
*/


Get all Gists
```
curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer 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" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/gists
```

Get specific Gist by ID
```
curl -L \
-H "Accept: application/vnd.github+json" \
-H "Authorization: Bearer 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" \
 -H "X-GitHub-Api-Version: 2022-11-28" \
https://api.github.com/gists/f1891d1e07f29ea9238d6ec656c467b8
```

Delete a file from a Gist
```
curl -X PATCH -H "Accept: application/vnd.github+json" -H "Authorization: Bearer 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" -d '{"files": {"FILE2.txt": null}}' https://api.github.com/gists/f1891d1e07f29ea9238d6ec656c467b8
```

Create Empty Gist
```
curl -X POST -H "Accept: application/vnd.github+json" -H "Authorization: Bearer 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" -d '{"description": "server", "public": false, "files":{"empty_file.txt":{"content":""}}}' https://api.github.com/gists
```