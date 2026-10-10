"""Use mature Dulwich for public Git checkouts; portable runtime needs no Git CLI."""
from pathlib import Path
from dulwich.repo import Repo
from dulwich import porcelain
from dulwich.client import get_transport_and_path

def revision(path):
    with Repo(str(path)) as repo:return repo.head().decode()

def origin(path):
    with Repo(str(path)) as repo:return repo.get_config().get((b'remote',b'origin'),b'url').decode()

def clean(path):
    status=porcelain.status(str(path))
    with Repo(str(path)) as repo:
        index=repo.open_index()
        dirty=[]
        for name in status.unstaged:
            canonical=name.replace(b'\\',b'/')
            try:entry=index[canonical]
            except KeyError:entry=None
            if entry and entry.skip_worktree and not (Path(path)/canonical.decode('utf-8')).exists():continue
            dirty.append(name)
    return not dirty and not any(status.staged.values())

def tracked(path):
    with Repo(str(path)) as repo:
        commit=repo[repo.head()]
        return {entry.path.decode('utf-8') for entry in repo.object_store.iter_tree_contents(commit.tree)}

def checkout(url,reference,target):
    target=Path(target)
    if target.exists():
        if origin(target).rstrip('/').removesuffix('.git')!=url.removesuffix('.git'):raise ValueError('Unexpected cache origin')
        if not clean(target):raise ValueError('Cache contains user modifications')
    else:
        target.mkdir(parents=True)
        repo=Repo.init(str(target));repo.close()
    with Repo(str(target)) as repo:
        config=repo.get_config();config.set((b'remote',b'origin'),b'url',url.encode());config.write_to_path()
        client,path=get_transport_and_path(url)
        wanted=reference.encode()
        def determine(refs,*args,**kwargs):
            if len(wanted)==40 and all(c in b'0123456789abcdef' for c in wanted):return [wanted]
            for name in (b'refs/tags/'+wanted+b'^{}',b'refs/tags/'+wanted,b'refs/heads/'+wanted):
                if name in refs:return [refs[name]]
            raise ValueError('Public reference not advertised')
        result=client.fetch(path,repo,determine_wants=determine,depth=1)
        selected=determine(result.refs)[0]
        while getattr(repo[selected],'type_name',b'')==b'tag':selected=repo[selected].object[1]
        repo.refs[b'HEAD']=selected
        repo.get_worktree().reset_index()
        return selected.decode()
