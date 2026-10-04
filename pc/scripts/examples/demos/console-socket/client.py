"""Tiny localhost client. python client.py 51707 demo_ping '= gd.player(1).x'

Start the game with MELEE_CONSOLE_PORT set to the chosen port. No game launch here.
Protocol credit: the project's pc/scripts/console.py and docs/scripting.md.
"""
import argparse
import socket


class Console:
    def __init__(self, port, timeout=10):
        self.sock=socket.create_connection(('127.0.0.1',port),timeout=timeout)
        self.file=self.sock.makefile('r',encoding='utf-8',errors='replace')
        try:
            if not self.file.readline():
                raise RuntimeError('console closed before its banner')
        except BaseException:
            self.close()
            raise

    def command(self, line):
        if '\n' in line or '\r' in line:
            raise ValueError('one command per line')
        self.sock.sendall((line+'\n').encode('utf-8'))
        reply=[]
        while True:
            text=self.file.readline()
            if not text:
                raise RuntimeError('console disconnected before terminator')
            text=text.rstrip('\r\n')
            if text=='>>> error':
                raise RuntimeError('\n'.join(reply) or 'console command failed')
            if text=='>>> ok':
                return reply
            reply.append(text)

    def close(self):
        self.file.close()
        self.sock.close()

    def __enter__(self):
        return self

    def __exit__(self,*args):
        self.close()


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('port',type=int)
    parser.add_argument('commands',nargs='*',default=['demo_ping','= gd.player(1).x'])
    args=parser.parse_args()
    try:
        with Console(args.port) as client:
            for command in args.commands:
                print('> '+command)
                print('\n'.join(client.command(command)))
    except (OSError,RuntimeError,ValueError) as exc:
        print('FAIL:',exc)
        return 1
    return 0


if __name__=='__main__':
    raise SystemExit(main())
