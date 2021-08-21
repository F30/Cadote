#!/usr/bin/env python3

import datetime
import os
import shutil
import subprocess
import time

import pandas as pd
import psutil


def main():

    if not os.path.samefile(os.getcwd(), os.path.dirname(__file__)):
        raise Exception('Must be run from source file directory')

    # TODO
    #ensure_users()
    #ensure_processes()
    #ensure_turbo_off()

    df = pd.DataFrame()

    df = df.append(build_unenclaved(), sort=True)
    df = df.append(build_enclaved(), sort=True)

    for i in range(5):
        unenclaved_result = run_password_check(False)
        df = df.append(unenclaved_result, sort=True)
        enclaved_result = run_password_check(True)
        df = df.append(enclaved_result, sort=True)

    print(df)

    print_msg('Writing serialized result to "measurement_password_check.tsv"')
    df.to_csv('measurement_password_check.tsv', sep='\t')


def ensure_users():

    who = subprocess.run(['who'], check=True, stdout=subprocess.PIPE)
    if who.stdout.count(b'\n') != 1:
        raise Exception('Too many logged in users')


def ensure_uptime():

    uptime = time.time() - psutil.boot_time()
    if uptime > 86400:
        raise Exception('Host not recently rebooted')


def ensure_processes():

    for p in psutil.process_iter():
        p_name = p.name()
        if 'cron' in p_name:
            raise Exception('cron must be stopped')
        if 'libvirtd' in p_name:
            raise Exception('libvirtd must be stopped')
        if 'snapd' in p_name:
            raise Exception('snapd must be stopped')
        if 'systemd-nspawn' in p_name:
            raise Exception('No systemd-nspawn processes must be running')
        if 'Runner.Listener' in p_name:
            raise Exception('github-actions-runner must be stopped')
        if p_name == 'sh' and 'vscode-server' in p.cmdline()[1]:
            raise Exception('No vscode-server processes must be running')

    # See https://unix.stackexchange.com/q/473992
    active_timers = subprocess.run(['systemctl', 'list-units', '--type=timer', '--state=active', '--no-legend'],
                                   check=True, stdout=subprocess.PIPE)
    if active_timers.stdout.count(b'\n') != 0:
        raise Exception('Found active systemd timers')

    # TODO: Adjust number
    if len(psutil.pids()) > 25:
        raise Exception('Too many processes running on system')


def ensure_turbo_off():

    with open('/sys/devices/system/cpu/intel_pstate/no_turbo') as f:
        no_turbo = f.read().rstrip()

    if no_turbo != '1':
        raise Exception('Turbo boost must be disabled')


def build_unenclaved():

    print_msg('Building unenclaved program in debug and release mode')
    os.symlink('Cargo.unenclaved.toml', 'Cargo.toml')

    debug_multiunit_cmd = ['cargo', 'build', '--features=enclavization_bin']
    debug_multiunit_result = {
        'program': 'build password_check',
        'operation': 'debug',
        'enclaved': False,
        'cmd': debug_multiunit_cmd,
        'descr': 'Time spent building the program in debug mode with multiple codegen units'
    }
    debug_1unit_cmd = ['cargo', 'rustc', '--features=enclavization_bin', '--', '-C', 'codegen-units=1']
    debug_1unit_result = {
        'program': 'build password_check',
        'operation': 'debug, codegen-units=1',
        'enclaved': False,
        'cmd': debug_1unit_cmd,
        'descr': 'Time spent building the program in debug mode with one codegen unit'
    }
    release_multiunit_cmd = ['cargo', 'build', '--features=enclavization_bin', '--release']
    release_multiunit_result = {
        'program': 'build password_check',
        'operation': 'release',
        'enclaved': False,
        'cmd': release_multiunit_cmd,
        'descr': 'Time spent building the program in release mode with multiple codegen units'
    }
    release_1unit_cmd = ['cargo', 'rustc', '--features=enclavization_bin', '--release', '--',
                         '-C', 'codegen-units=1']
    release_1unit_result = {
        'program': 'build password_check',
        'operation': 'release, codegen-units=1',
        'enclaved': False,
        'cmd': release_1unit_cmd,
        'descr': 'Time spent building the program in release mode with one codegen unit'
    }

    try:
        shutil.rmtree('target', ignore_errors=True)
        begin = time.time()
        subprocess.run(debug_multiunit_cmd, check=True)
        debug_multiunit_result['duration'] = (time.time() - begin) * 1000
        debug_multiunit_result['time'] = datetime.datetime.now()

        shutil.rmtree('target', ignore_errors=True)
        begin = time.time()
        subprocess.run(debug_1unit_cmd, check=True)
        debug_1unit_result['duration'] = (time.time() - begin) * 1000
        debug_1unit_result['time'] = datetime.datetime.now()

        shutil.rmtree('target', ignore_errors=True)
        begin = time.time()
        subprocess.run(release_multiunit_cmd, check=True)
        release_multiunit_result['duration'] = (time.time() - begin) * 1000
        release_multiunit_result['time'] = datetime.datetime.now()

        shutil.rmtree('target', ignore_errors=True)
        begin = time.time()
        subprocess.run(release_1unit_cmd, check=True)
        release_1unit_result['duration'] = (time.time() - begin) * 1000
        release_1unit_result['time'] = datetime.datetime.now()

        return [debug_multiunit_result, debug_1unit_result, release_multiunit_result, release_1unit_result]
    finally:
        os.remove('Cargo.toml')


def build_enclaved():

    print_msg('Building enclaved program in debug and release mode')

    cmd = ['make', 'all']
    debug_result = {
        'program': 'build password_check',
        'operation': 'debug, codegen-units=1',
        'enclaved': True,
        'cmd': cmd,
        'descr': 'Time spent building the program in debug mode'
    }
    release_result = {
        'program': 'build password_check',
        'operation': 'release, codegen-units=1',
        'enclaved': True,
        'cmd': cmd,
        'descr': 'Time spent building the program in release mode'
    }

    subprocess.run(['make', 'clean'], check=True)
    begin = time.time()
    subprocess.run(cmd, check=True)
    debug_result['duration'] = (time.time() - begin) * 1000
    debug_result['time'] = datetime.datetime.now()

    subprocess.run(['make', 'clean'], check=True)
    begin = time.time()
    subprocess.run(cmd, env=dict(os.environ, BUILD_MODE='release'), check=True)
    release_result['duration'] = (time.time() - begin) * 1000
    release_result['time'] = datetime.datetime.now()

    return [debug_result, release_result]


def run_password_check(enclaved):

    try:
        os.remove('users.shadow')
    except FileNotFoundError:
        pass

    results = []

    if enclaved:
        password_check_path = 'build/password_check'
        print_msg('Running enclaved password_check sequence')
    else:
        password_check_path = 'target/release/password_check'
        print_msg('Running unenclaved password_check sequence')

    initalize_cmd = [password_check_path]
    initialize_input = 'topsecret\nq\n'.encode('utf8')
    initialize_result = {
        'program': 'password_check',
        'operation': 'initialize',
        'enclaved': enclaved,
        'cmd': initalize_cmd,
        'descr': 'Time from after program startup to having created an initial root user',
        'time': datetime.datetime.now()
    }
    initialize = subprocess.run(initalize_cmd, input=initialize_input, stdout=subprocess.PIPE,
                                stderr=subprocess.PIPE, check=True)
    if 'Stored.' not in initialize.stdout.decode('utf8'):
        raise Exception('Unexpected initalization output')
    initialize_result['duration'] = get_duration_millis(initialize.stderr)
    results.append(initialize_result)

    add_first_cmd = [password_check_path]
    add_first_input = 'root\ntopsecret\na\nalice\n12345678\nq\n'.encode('utf8')
    add_first_result = {
        'program': 'password_check',
        'operation': 'add',
        'enclaved': enclaved,
        'size': 1,
        'cmd': add_first_cmd,
        'descr': 'Time from issuing the add user command to having stored the first user',
        'time': datetime.datetime.now()
    }
    add_first = subprocess.run(initalize_cmd, input=add_first_input, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, check=True)
    add_first_out = add_first.stdout.decode('utf8')
    if 'Authenticated successfully!' not in add_first_out or 'Stored.' not in add_first_out:
        raise Exception('Unexpected adding output')
    add_first_result['duration'] = get_duration_millis(add_first.stderr)
    results.append(add_first_result)

    login_first_cmd = [password_check_path]
    login_first_input = 'alice\n12345678\n'.encode('utf8')
    login_first_result = {
        'program': 'password_check',
        'operation': 'login',
        'enclaved': enclaved,
        'size': 1,
        'cmd': login_first_cmd,
        'descr': 'Time from after program startup to having logged in as first user',
        'time': datetime.datetime.now()
    }
    login_first = subprocess.run(login_first_cmd, input=login_first_input, stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, check=True)
    if 'Authenticated successfully!' not in login_first.stdout.decode('utf8'):
        raise Exception('Unexpected login output')
    login_first_result['duration'] = get_duration_millis(login_first.stderr)
    results.append(login_first_result)

    for i in range(98):
        fillup_input = 'root\ntopsecret\na\nuser{:d}\npassword\nq\n'.format(i).encode('utf8')
        subprocess.run([password_check_path], input=fillup_input, stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL, check=True)

    add_jubilee_cmd = [password_check_path]
    add_jubilee_input = 'root\ntopsecret\na\njubilee\ndrowssap\nq\n'.encode('utf8')
    add_jubilee_result = {
        'program': 'password_check',
        'operation': 'add',
        'enclaved': enclaved,
        'size': 100,
        'cmd': add_jubilee_cmd,
        'descr': 'Time from issuing the add user command to having stored the 100th user',
        'time': datetime.datetime.now()
    }
    add_jubilee = subprocess.run(add_jubilee_cmd, input=add_jubilee_input, stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, check=True)
    add_jubilee_out = add_jubilee.stdout.decode('utf8')
    if 'Authenticated successfully!' not in add_jubilee_out or 'Stored.' not in add_jubilee_out:
        raise Exception('Unexpected adding output')
    add_jubilee_result['duration'] = get_duration_millis(add_jubilee.stderr)
    results.append(add_jubilee_result)

    login_jubilee_cmd = [password_check_path]
    login_jubilee_input = 'jubilee\ndrowssap\n'.encode('utf8')
    login_jubilee_result = {
        'program': 'password_check',
        'operation': 'login',
        'enclaved': enclaved,
        'size': 100,
        'cmd': login_jubilee_cmd,
        'descr': 'Time from after program startup to having logged in as 100th user',
        'time': datetime.datetime.now()
    }
    login_jubilee = subprocess.run(login_jubilee_cmd, input=login_jubilee_input, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, check=True)
    if 'Authenticated successfully!' not in login_jubilee.stdout.decode('utf8'):
        raise Exception('Unexpected login output')
    login_jubilee_result['duration'] = get_duration_millis(login_jubilee.stderr)
    results.append(login_jubilee_result)

    return results


def print_msg(message):

    print('>>> {}'.format(message))


def get_duration_millis(stderr_data):

    duration_prefix = 'EVALUATION DURATION: '

    for line in stderr_data.decode('utf8').split('\n'):
        if line.startswith(duration_prefix):
            duration_micros = line[len(duration_prefix):]
            return int(duration_micros) / 1000

    raise Exception('Could not determine duration')


if __name__ == '__main__':

    main()
