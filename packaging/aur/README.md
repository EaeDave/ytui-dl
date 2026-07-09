# Publicar ytui-dl no AUR

O AUR **não** usa o repositório do GitHub do app.  
É um **repositório Git separado** no servidor do AUR, contendo só:

- `PKGBUILD`
- `.SRCINFO`
- (opcional) patches, `.install`, etc.

## 1. Conta e SSH (uma vez)

1. Crie conta em https://aur.archlinux.org/register  
2. Gere uma chave **só para o AUR** (recomendado):

```bash
ssh-keygen -f ~/.ssh/aur -t ed25519 -C "aur"
```

3. Em **My Account** no AUR, cole o conteúdo de `~/.ssh/aur.pub`.  
4. Configure o SSH:

```bash
# ~/.ssh/config
Host aur.archlinux.org
  IdentityFile ~/.ssh/aur
  User aur
```

5. Teste:

```bash
ssh aur@aur.archlinux.org help
```

## 2. Qual pacote publicar primeiro?

| Pacote | Quando usar |
|--------|-------------|
| **`ytui-dl-git`** | Build a partir do `main` do GitHub. Ideal no início. |
| **`ytui-dl`** | Release estável com tag `v0.1.0` + checksums. |

Comece com **`ytui-dl-git`** (PKGBUILD já está neste diretório).

## 3. Publicar `ytui-dl-git`

```bash
# clone do “repo vazio” no AUR (aviso de empty é normal)
git -c init.defaultBranch=master clone ssh://aur@aur.archlinux.org/ytui-dl-git.git
cd ytui-dl-git

# copie o PKGBUILD do projeto
cp /caminho/para/ytui-dl/packaging/aur/ytui-dl-git/PKGBUILD .

# edite o e-mail do Maintainer no topo do PKGBUILD

# teste local (demora: baixa deps e compila)
makepkg -si

# se ok, gere metadados e envie
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Initial import: ytui-dl-git"
git push
```

Depois de uns minutos o pacote aparece em:

https://aur.archlinux.org/packages/ytui-dl-git

Usuários instalam com:

```bash
yay -S ytui-dl-git
# ou
paru -S ytui-dl-git
```

## 4. Atualizar o pacote

Quando mudar o build/deps (não a cada commit do app no `-git`):

```bash
# edite PKGBUILD se preciso
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Update build / deps"
git push
```

No pacote **estável** (`ytui-dl`), a cada release:

1. Tag no GitHub: `git tag v0.1.1 && git push --tags`
2. Suba `pkgver`, recalcule `sha256sums` (`updpkgsums` ou `makepkg -g`)
3. Bumpe `pkgrel` só se mudar o PKGBUILD sem mudar versão do app
4. Regenere `.SRCINFO`, commit, push

## 5. Manutenção

- Responda comentários no AUR  
- Não abandone o pacote sem **disown**  
- Não comente “updated to x.y” a cada bump — use o commit/changelog do AUR  

## Referências

- https://wiki.archlinux.org/title/AUR_submission_guidelines  
- https://wiki.archlinux.org/title/Rust_package_guidelines  
- https://wiki.archlinux.org/title/VCS_package_guidelines  
