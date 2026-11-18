"""Use the installed official LBPPC browser sign-in and game-file checker locally."""
import argparse
import base64
import ctypes
import json
import os
from pathlib import Path
import sys
import tempfile
import webbrowser


def protect(data, decrypt=False):
    if os.name != 'nt':
        raise RuntimeError('Windows account protection is required for sign-in storage.')
    class Blob(ctypes.Structure):
        _fields_ = [('length', ctypes.c_ulong), ('pointer', ctypes.c_void_p)]
    buffer = ctypes.create_string_buffer(data)
    source = Blob(len(data), ctypes.cast(buffer, ctypes.c_void_p))
    result = Blob()
    library = ctypes.WinDLL('crypt32', use_last_error=True)
    function = library.CryptUnprotectData if decrypt else library.CryptProtectData
    function.argtypes = [ctypes.POINTER(Blob), ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_ulong, ctypes.POINTER(Blob)]
    if not function(ctypes.byref(source), None, None, None, None, 1, ctypes.byref(result)):
        raise RuntimeError('Windows could not unlock/store this sign-in. Sign in again on this Windows account.')
    try:
        return ctypes.string_at(result.pointer, result.length)
    finally:
        free = ctypes.WinDLL('kernel32').LocalFree
        free.argtypes = [ctypes.c_void_p]
        free(result.pointer)


def read_settings(path):
    if not path.exists():
        return {}
    if path.stat().st_size > 2 * 1024 * 1024:
        raise RuntimeError('LBPPC settings exceed the supported size.')
    data = json.loads(path.read_text(encoding='utf-8-sig'))
    if not isinstance(data, dict):
        raise RuntimeError('LBPPC settings are not an object; they were not replaced.')
    return data


def write_settings(path, changes, remove=()):
    values = read_settings(path)
    values.update(changes)
    for key in remove:
        values.pop(key, None)
    path.parent.mkdir(parents=True, exist_ok=True)
    handle, temporary = tempfile.mkstemp(dir=path.parent, prefix='.pod-signin-', suffix='.json')
    try:
        with os.fdopen(handle, 'w', encoding='utf-8') as stream:
            json.dump(values, stream, indent=2)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def stored_signin(settings):
    locked = settings.get('lbppc_signin_protected')
    if not locked:
        return None
    try:
        value = json.loads(protect(base64.b64decode(locked, validate=True), decrypt=True))
        return value if isinstance(value, dict) and value.get('refresh_token') else None
    except (ValueError, RuntimeError):
        return None


def operate(root, action, cancel=None, game=None, update=None):
    settings_file = root / 'user/launcher.json'
    settings = read_settings(settings_file)
    session = stored_signin(settings)
    if action == 'status':
        return {'signedIn': bool(session), 'username': (session or {}).get('username', ''), 'online': bool(settings.get('online'))}
    sys.path.insert(0, str(root / 'scripts'))
    if action == 'check':
        import game_files
        import game_setup
        selected_update = Path(update) if update else game_setup.find_update(Path(game))
        result = game_files.check(Path(game), update=selected_update)
        return {'complete': result.ok, 'message': result.summary(False), 'detectedUpdate': str(selected_update) if selected_update else ''}
    import lbppc_signin
    def save(value):
        if value is None:
            write_settings(settings_file, {'online': False}, ('lbppc_signin', 'lbppc_signin_protected'))
        else:
            locked = base64.b64encode(protect(json.dumps(value).encode('utf-8'))).decode('ascii')
            write_settings(settings_file, {'lbppc_signin': None, 'lbppc_signin_protected': locked,
                                           'online': True, 'server': 'play.lbppc.com', 'online_name': value.get('username', '')})
    account = lbppc_signin.Account(load=lambda: session, save=save)
    if action == 'signin':
        tokens = lbppc_signin.browser_sign_in(webbrowser.open, cancelled=lambda: bool(cancel and cancel.exists()))
        account.signed_in_with(tokens)
        return {'signedIn': True, 'username': account.username or '', 'online': True}
    if action == 'signout':
        account.sign_out()
        return {'signedIn': False, 'username': '', 'online': False}
    raise RuntimeError('Unknown LBPPC operation.')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--action', choices=['status', 'signin', 'signout', 'check'], required=True)
    parser.add_argument('--cancel', type=Path)
    parser.add_argument('--game')
    parser.add_argument('--update')
    parser.add_argument('--result', type=Path)
    args = parser.parse_args()
    try:
        result = operate(args.root, args.action, args.cancel, args.game, args.update)
        response = {'ok': True, **result}
    except Exception:
        # Provider errors can contain URLs/codes. Do not return secrets or raw tracebacks.
        response = {'ok': False, 'message': 'LBPPC setup/sign-in could not finish. It may have been cancelled, the server may be unavailable, or local settings may need attention. Use the official LBPPC settings for details.'}
    if args.result:
        args.result.write_text(json.dumps(response), encoding='utf-8')
    elif sys.stdout is not None:
        print(json.dumps(response), flush=True)


if __name__ == '__main__':
    main()
