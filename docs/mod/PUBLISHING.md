# Publish the source repository

The delivered source ZIP includes an initialized local Git repository on branch `main`, with the source export commit and tag `weapon-evolution-v0.1.6.39`, followed by a developer-documentation update tagged `source-docs-v1`. It has no remote and contains no GitHub credentials. The prepared commits use the generic local identity `Source Export <source-export@localhost>`; configure your own identity for future work.

## GitHub Desktop

1. Extract the entire ZIP, keeping the repository contents together.
2. In GitHub Desktop, choose **File > Add local repository** and select the extracted `rerac-weapon-evolution` folder.
3. Choose **Publish repository**, choose the name and your account, and set the desired visibility. Clear **Keep this code private** if you want a public source repository.
4. Publish. The generated player ZIP belongs in a GitHub Release; the source repository excludes executable builds and local game data.

Official instructions: https://docs.github.com/en/desktop/adding-and-cloning-repositories/adding-an-existing-project-to-github-using-github-desktop

## Git command line

Create an empty repository under your GitHub account. Avoid initializing that remote with an additional README or license because this export already contains them. In the extracted local repository, replace the example URL with your own:

```sh
git remote add origin https://github.com/YOUR_ACCOUNT/rerac-weapon-evolution.git
git push -u origin main
git push origin weapon-evolution-v0.1.6.39
git push origin source-docs-v1
```

The bundled tag and commit describe the prepared source snapshot. They do not recreate historical upstream commits or claim authorship of ReRAC. Preserve the supplied license and credits when publishing.

If you already published the initial export, preserve your existing checkout's `.git` folder and remote settings when copying updated documentation into it, then commit and push normally. The documentation update adds no runtime gameplay changes.

Local presets and `weapons.ini` are ignored. To update the public default intentionally, edit the tracked default/example INIs together and check the optional embedded profile. Run the source checker and tests before committing.
