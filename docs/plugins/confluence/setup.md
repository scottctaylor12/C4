# Confluence: Setup

1. Sign up for a free Atlassian Confluence account

2. Create a new Confluence *"Space"* to publish pages to. These pages will be where the C2 messages are posted at. Remember the shortname of the space that you made (Example: "TESTING" may have gotten shortened to "TEST"). You should be able to browse to this newly created space by going to:
`https://<domain>.atlassian.net/wiki/spaces/<NAME>/`

3. Create an API Key
    1. In the upper right corner of Confluence, click profile and select `Account Settings` in the dropdown

    2. In the Account Settings, select `Security`

    3. Find the *API tokens* section and select `Create and manage API tokens`. (You may have to two-factor authenticate)

    4. Select `Create API token`, give your token a name, and specify an expiration date long enough for your needs.

    5. **CRUCIAL!** Copy the API key and store it somewhere safe. This key gives full programatic access to your Atlassian account, so don't accidentally leak it.

By following the steps above, you should have:
* Confluence account with a URL such as `https://<domain>.atlassian.net/wiki`
* Confluence space created in the account (remember the short name)
* Atlassian API key
