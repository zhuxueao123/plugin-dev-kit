# Plugin project development

This project is independent of the tool repository. Developers manage its Git repository.
Use `asapflow plugin init --root . --code <code> --name <name> --kind full` to create a plugin.
Use `asapflow plugin validate`, `pack`, and `publish` with `--root . --code <code>`.
Frontend dependencies: run `npm install` in frontend/ before building.
Read plugin-development/ for API and SDK references and cli/ for command details.
Documentation is managed under docs/dev-kit/; project-specific instructions belong in docs/project/.
Check updates with `asapflow plugin init-project --root . --check`.
Apply updates with `asapflow plugin init-project --root . --update`.
Existing plugin code and project Git are never managed by the tool kit.
