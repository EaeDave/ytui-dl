# Publicar ytd no AUR

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
| **`ytd-git`** | Build a partir do `main` do GitHub. Ideal no início. |
| **`ytd`** | Release estável com tag + checksums. |

Comece com **`ytd-git`** (PKGBUILD já está neste diretório).

## 3. Publicar `ytd-git`

```bash
# clone do “repo vazio” no AUR (aviso de empty é normal)
git -c init.defaultBranch=master clone ssh://aur@aur.archlinux.org/ytd-git.git
cd ytd-git

# copie o PKGBUILD do projeto
cp /caminho/para/ytd/packaging/aur/ytd-git/PKGBUILD .

# edite o e-mail do Maintainer no topo do PKGBUILD

# teste local (demora: baixa deps e compila)
makepkg -si

# se ok, gere metadados e envie
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Initial import: ytd-git"
git push
```

Depois de uns minutos o pacote aparece em:

https://aur.archlinux.org/packages/ytd-git

Usuários instalam com:

```bash
yay -S ytd-git
# ou
paru -S ytd-git
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

No pacote **estável** (`ytd`), a cada release:

1. Tag no GitHub: `git tag v0.5.1 && git push --tags`
2. Suba `pkgver`, recalcule `sha256sums` (`updpkgsums` ou `makepkg -g`)
3. Bumpe `pkgrel` só se mudar o PKGBUILD sem mudar versão do app
4. Regenere `.SRCINFO`, commit, push

## 5. Manutenção

- Responda comentários no AUR  
- Não abandone o pacote sem **disown**  
