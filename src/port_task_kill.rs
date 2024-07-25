use std::{io, process};
use std::collections::HashSet;
use std::io::Write;
use std::process::Output;
use std::thread::sleep;
use std::time::Duration;

use netstat::*;

/**
 * 结束端口对应进程
 */
pub fn port_kill() {
    print!("请输入端口号（用空格分隔）：");
    io::stdout().flush().unwrap();
    let console_result = read_console();
    if console_result.is_err() {
        process_exit();
    }

    let ports = console_result.unwrap();
    if ports.is_empty() {
        println!("未读取到有效端口号！");
        process_exit();
    }

    let pids: HashSet<u32> = get_process_number(ports);

    if !pids.is_empty() {
        println!("即将杀死进程：{:?}", pids);
        process_kill(pids);
    } else {
        println!("未找到对应端口进程！");
    }

    process_exit()
}

fn process_kill(pids: HashSet<u32>) {
    let func: fn(u32) -> std::io::Result<Output> = if cfg!(target_os = "windows") {
        |pid| windows_kill(pid)
    } else if cfg!(target_os = "linux") {
        |pid| linux_kill(pid)
    } else {
        println!("操作系统识别错误！");
        process_exit();
    };

    for pid in pids {
        let result = func(pid);
        print_process_kill_result(pid, result);
    }
}

fn print_process_kill_result(pid: u32, result: std::io::Result<Output>) {
    match result {
        Ok(_) => println!("{} 已杀死！", pid),
        Err(e) => println!("无法杀死进程 {}，异常：{}！", pid, e),
    }
}

fn windows_kill(pid: u32) -> std::io::Result<Output> {
    let kill_result: std::io::Result<Output> = process::Command::new("taskkill")
        .arg("/F")
        .arg("/PID")
        .arg(pid.to_string())
        .output();
    kill_result
}
fn linux_kill(pid: u32) -> std::io::Result<Output> {
    println!("{}",pid.to_string());
    let kill_result: std::io::Result<Output> = process::Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .output();
    kill_result
}

fn read_console() -> Result<Vec<u16>, io::Error> {
    let mut buf = String::new();
    let result: std::io::Result<usize> = io::stdin().read_line(&mut buf);
    match result {
        Err(e) => {
            println!("参数读取异常！{}", e);
            return Err(e);
        }
        _ => {}
    }
    let ports: Vec<u16> = buf.as_str()
        .split_whitespace()
        .map(|item| item.parse::<u16>())
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap())
        .collect();

    Ok(ports)
}

fn get_process_number(port: Vec<u16>) -> HashSet<u32> {
    let af_flags: AddressFamilyFlags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let sockets_info = get_sockets_info(af_flags, proto_flags).unwrap();
    let mut pids: HashSet<u32> = HashSet::new();
    for si in sockets_info {
        match si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_si) => {
                if port.contains(&tcp_si.local_port) {
                    let pid = si.associated_pids;
                    pid.into_iter().for_each(|i| {
                        pids.insert(i);
                    });
                }
            }
            _ => {}
        }
    }
    pids
}

fn process_exit() -> ! {
    println!("任务执行结束，将在三秒后退出");
    sleep(Duration::from_secs(3));
    process::exit(0);
}
