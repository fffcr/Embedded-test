# DM-MC02 USB 调试器作业 · 详细执行指南

> 适用工程：`Embedded-test`  
> 目标芯片：STM32H723VGT6  
> 参考：项目 `README.md`、达妙原理图 V1.0、使用说明书 V1.1、`考核指导/DM-MC02 考核指南.md`  
> 当前状态（2026-11-13）：`probe-rs 0.32.0` 已装；`.venv` + `pyserial 3.5` 已装；`src/wiring.rs` 已实现 PE9/PE13 + UART7；当前代码已在临时副本编译通过；剩余工作是重新编译、烧录、实测和记录。

---

## 目录

- [0. 这份指南怎么用](#0-这份指南怎么用)
- [1. 作业目标与验收标准](#1-作业目标与验收标准)
- [2. 原理速览](#2-原理速览)
- [3. 名词解释](#3-名词解释)
- [4. 硬件准备与接线](#4-硬件准备与接线)
- [5. Step 0：检查环境](#5-step-0检查环境)
- [6. Step 1：编译工程](#6-step-1编译工程)
- [7. Step 2：可选——先烧原始模板](#7-step-2可选先烧原始模板)
- [8. Step 3：确认引脚选择](#8-step-3确认引脚选择)
- [9. Step 4：实现 TODO 1（SWD GPIO）](#9-step-4实现-todo-1swd-gpio)
- [10. Step 5：实现 TODO 2（Buffered UART）](#10-step-5实现-todo-2buffered-uart)
- [11. Step 6：完整编译](#11-step-6完整编译)
- [12. Step 7：烧录新版](#12-step-7烧录新版)
- [13. Step 8：上电与接测试跳线](#13-step-8上电与接测试跳线)
- [14. Step 9：USB 枚举与端口识别](#14-step-9usb-枚举与端口识别)
- [15. Step 10：SWD 验收](#15-step-10swd-验收)
- [16. Step 11：UART 回环验收](#16-step-11uart-回环验收)
- [17. Step 12：选做 CAN](#17-step-12选做-can)
- [18. Step 13：记录与提交](#18-step-13记录与提交)
- [19. 故障排查总表](#19-故障排查总表)
- [20. 附录 A：`wiring.rs` 参考实现](#20-附录-awiringrs-参考实现)
- [21. 附录 B：源码导读](#21-附录-b源码导读)
- [22. 附录 C：引脚速查](#22-附录-c引脚速查)
- [23. 附录 D：命令速查卡](#23-附录-d命令速查卡)
- [24. 附录 E：术语表](#24-附录-e术语表)

---

## 0. 这份指南怎么用

### 0.1 阅读顺序

从 Step 0 一直做到 Step 13。每一步都按同样的结构写：

1. **做什么**：具体命令或操作。
2. **为什么**：这一步在解决什么问题。
3. **预期**：正常情况下应该看到什么。
4. **卡住了**：对应故障排查表的位置。

### 0.2 三种标记

> 📌 **注解**：名词或背景解释。第一遍可以跳过，遇到不懂再回来看。

> ⚠️ **注意**：接错可能烧板子、烧探针或让作业失败的地方。

> ✅ **检查点**：做到这里应该停下来确认结果，再继续。

### 0.3 命令里的占位符

```text
$CONTROL   = 控制串口的实际设备名，例如 /dev/cu.usbmodem12301
$UART      = UART 串口的实际设备名，例如 /dev/cu.usbmodem12303
```

每台电脑、每次插拔，编号都可能变化。**不要照抄示例里的编号**。

### 0.4 当前进度

| 项目 | 状态 |
| --- | --- |
| Rust 工具链 1.99.1 + `thumbv7em-none-eabihf` | ✅ 已就绪 |
| `probe-rs` | ✅ 0.32.0 |
| `.venv` + `pyserial` | ✅ 3.5 |
| `src/wiring.rs` 两个 TODO | ✅ 已写（PE9/PE13 + UART7） |
| 编译 | ✅ 临时副本 `cargo build --locked` 通过 |
| 烧录 | ⬜ 待做 |
| SWD 实测 | ⬜ 待做 |
| UART 回环实测 | ⬜ 待做 |
| 实测记录与提交 | ⬜ 待做 |

---

## 1. 作业目标与验收标准

### 1.1 这个作业到底在做什么

把 DM-MC02 做成一个「三合一 USB 设备」：

```text
电脑
 │
 │ USB-C
 ▼
DM-MC02（STM32H723VG 跑你的固件）
 ├─ ① 控制串口：接收 PING / SWD ID / CAN 命令
 ├─ ② UART 串口：USB ↔ UART7 的原始字节透传
 └─ ③ CMSIS-DAP v2 演示接口：让 probe-rs 能“发现”这块板子
        │
        │ 你配置的 PE9 / PE13
        ▼
     P1 调试口 → 目标芯片的 SWD
```

三条通路里，**只有第 ③ 条是“假的探针”**：它只回答设备信息，不能真的烧录。真正的 SWD 通信是第 ① 条里的 `SWD ID` 命令，用两个 GPIO 手动打时序完成的。

### 1.2 你要交什么

按 `README.md` 的「提交」一节：

1. `src/wiring.rs`：补完两个 TODO。
2. **引脚表**：写清 SWCLK、SWDIO、UART TX、UART RX 的 MCU 引脚和接插件脚号。
3. **实测记录**：测试命令、实际结果、未测项目，以及一项用实测确认的配置。

### 1.3 验收标准

| 验收项 | 命令 | 通过标准 |
| --- | --- | --- |
| USB 演示接口被发现 | `probe-rs list` | 列出 `DM-MC02 CMSIS-DAP Demo` |
| 控制口存活 | `python3 host.py $CONTROL ping` | `OK DM-MC02` |
| SWD 能读 ID | `python3 host.py $CONTROL swd` | `OK DPIDR XXXXXXXX`，末尾 8 位十六进制 |
| SWD 稳定 | 再执行一次 `swd` | 两次 DPIDR 完全相同 |
| MCU 没被搞死 | 再执行 `ping` | 再次 `OK DM-MC02` |
| UART 通 | `python3 host.py $UART uart-loopback` | `OK UART loopback` |

> 📌 **注解 · 为什么验收要分两层？**  
> `probe-rs list` 只能证明 USB 描述符写对了；`SWD ID` 才能证明你选的两个 GPIO、方向切换和时序真的能跟目标芯片通信。README 里专门写了「标准探针接口只验收发现，实际 SWD 读取用 `SWD ID`」。

---

## 2. 原理速览

### 2.1 从上电到程序运行

```text
复位
  │
  ├─ 从 Flash 起始地址 0x08000000 取向量表
  ├─ cortex-m-rt 的 Reset handler：
  │    清 .bss、把 .data 从 Flash 拷到 RAM
  ├─ 进入 #[embassy_executor::main] 生成的入口
  ├─ 初始化 embassy 执行器和 TIM2 时间驱动
  └─ 调用你写的 async fn main()
```

> 📌 **注解 · Flash 和 SRAM**  
> Flash 相当于「硬盘」：程序烧进去后断电不丢。SRAM 相当于「内存」：程序运行时变量放这里，断电就没了。`memory.x` 里写的是：
>
> ```text
> FLASH: 0x08000000，1 MB
> RAM  : 0x24000000，128 KB
> ```

### 2.2 程序怎么进芯片

本作业第一次烧录用的是**外置调试探针**：

```text
电脑（probe-rs）
   │ USB
   ▼
外置探针（CMSIS-DAP / J-Link 等）
   │ SWD：SWCLK + SWDIO + GND
   ▼
DM-MC02 的 P1 调试口
   │
   ▼
STM32H723 内部 Flash
```

烧好以后，DM-MC02 自己就能用 GPIO 去读目标芯片的调试端口 ID。

> 📌 **注解 · DFU 是什么？**  
> DFU（Device Firmware Upgrade）是 ST 预装在芯片系统存储器里的 Bootloader。把 BOOT0 拉高再复位，芯片会进入 DFU 模式，电脑可以用 `dfu-util` 之类的工具通过 USB 烧录。这是另一条路，本作业不用，但原理图里的 P14/BOOT0 就是干这个的。

### 2.3 调试链路的三个角色

| 角色 | 在本作业里是谁 | 作用 |
| --- | --- | --- |
| 主机工具 | 电脑上的 `probe-rs` / OpenOCD | 发调试命令 |
| 探针 | 外置探针（烧录时）；DM-MC02 固件（自测时） | 把 USB 命令变成 SWD 电信号 |
| 目标 | DM-MC02 自己的 STM32 | 芯片内置的调试端口 DAP |

```text
电脑（probe-rs / OpenOCD）
        ↕ USB 上的 CMSIS-DAP v2
USB 调试探针
        ↕ SWCLK / SWDIO
目标 STM32（芯片内置的 DAP）
```

### 2.4 同板自测为什么可行

你会想：DM-MC02 用自己的 GPIO 去读自己的调试口，这不是「自己搞自己」吗？

不是。芯片里的 **DAP（Debug Access Port）是独立硬件**。CPU 在跑你的 USB 固件时，DAP 仍然可以被外部 SWD 信号访问。真实的调试器也是这么工作的：CPU 不停机，调试器照样能读寄存器和内存。

你的 PE9/PE13 只是扮演了「外部调试器」的电信号：

```text
PE9  ──跳线── P1.3 ── PA14（SWCLK）
PE13 ──跳线── P1.4 ── PA13（SWDIO）
```

### 2.5 USB 上的三条通路

| USB 接口 | 系统里看到的东西 | 固件里的处理 |
| --- | --- | --- |
| CDC ACM #1 | 一个串口 | `control_port` 解析 ASCII 命令 |
| CDC ACM #2 | 另一个串口 | `uart_port` 转发原始字节到 UART7 |
| 厂商自定义 0xFF | 一个 CMSIS-DAP v2 接口 | `dap_usb::run` 只回答设备信息 |

> 📌 **注解 · CDC ACM**  
> CDC ACM（Communications Device Class Abstract Control Model）就是「USB 虚拟串口」。电脑会把它当成一个普通的串口设备，但板子上并没有对应的物理 UART——数据是固件在 USB 和真实外设之间搬的。

### 2.6 一张总图

```text
┌────────────────────────── 电脑 ──────────────────────────┐
│  probe-rs        host.py / pyserial                      │
└──────┬───────────────────┬───────────────────────────────┘
       │ USB               │ USB
       ▼                   ▼
┌──────────────────────────────────────────────────────────┐
│                    DM-MC02（STM32H723）                   │
│                                                          │
│  USB 设备栈                                               │
│   ├─ CDC #1 → control_port → protocol → CAN / SWD        │
│   ├─ CDC #2 → uart_port ────────────────┐                │
│   └─ CMSIS-DAP → dap_usb（只回 DAP_Info）│                │
│                                          │                │
│  UART7 (PE7/PE8) ◄───────────────────────┘                │
│  SWD GPIO (PE9/PE13) ──┐                                 │
└─────────────────────────┼────────────────────────────────┘
                          │ 跳线
                          ▼
                     P1.3 / P1.4
                          │
                          ▼
                  目标芯片的 DAP（同板或另一块板）
```

---

## 3. 名词解释

这一节按主题列出会用到的名词。第一次读可以只看加粗部分，遇到不懂再回来查。

### 3.1 芯片与外设

| 名词 | 直白解释 |
| --- | --- |
| **MCU** | 微控制器。把 CPU、Flash、SRAM、GPIO、UART、USB 等集成在一颗芯片里。 |
| **STM32H723VG** | 本板的 MCU 型号，Cortex-M7 内核，1 MB Flash，564 KB SRAM。 |
| **GPIO** | 通用输入输出引脚。可以当开关用：输出高/低电平，或读入外部电平。 |
| **外设** | 芯片里除 CPU 以外的功能模块，例如 UART、USB、CAN、定时器。 |
| **AF（Alternate Function）** | 复用功能。一个引脚可以当 GPIO，也可以交给 UART/SPI/定时器使用；AF 编号决定它归谁。 |
| **推挽输出（push-pull）** | 引脚既能主动拉高，也能主动拉低。`Output` 默认就是这种。 |
| **高阻/输入** | 引脚不驱动，只“听”外面的电平。双向总线在“让别人说话”时必须切成输入。 |
| **上拉（pull-up）** | 内部电阻把线轻轻拉到高电平，避免悬空时电平乱跳。 |
| **3.3 V GPIO** | 本作业要求选 3.3 V 信号脚；不要接 5 V。 |
| **5 V 容忍** | 某些引脚即使供电 3.3 V，也能承受 5 V 输入。PA13/PA14 是 5 V 容忍，但这不代表所有 PE 引脚都是。 |

> 📌 **注解 · 为什么 SWDIO 必须能切换方向？**  
> SWD 只有一根数据线。主机发请求时由主机驱动；目标回 ACK 和数据时，主机必须“松手”，否则两个输出同时拉线，会打架。`Flex` 就是能切换「驱动 / 监听」的引脚类型。

### 3.2 UART

| 名词 | 直白解释 |
| --- | --- |
| **UART** | 异步串口。两根线：TX 发、RX 收，没有时钟线，双方靠约定好的波特率。 |
| **USART** | 比 UART 多同步模式等功能；本板 UART7 是 UART。 |
| **TX** | Transmit，发送。本作业 UART7 TX = PE8。 |
| **RX** | Receive，接收。本作业 UART7 RX = PE7。 |
| **波特率（baud）** | 每秒传输多少位。115200 表示每秒约 115200 个位。 |
| **8N1** | 8 个数据位、无校验（None）、1 个停止位。加 1 个起始位，共 10 位/字节。 |
| **全双工** | TX 和 RX 可以同时工作。 |
| **中断（interrupt）** | 外设有事时“按门铃”打断 CPU。UART 收到字节、USB 收到包、CAN 收到帧都会触发中断。 |
| **ISR** | Interrupt Service Routine，中断服务函数。 |
| **缓冲区（buffer）** | 一块临时存放数据的内存。中断把数据放进去，主程序再慢慢取。 |
| **环形缓冲区** | 首尾相接的缓冲区；写满会绕回开头。`BufferedUart` 内部用的就是这种。 |
| **溢出（overrun）** | 数据来得太快，缓冲区满了，新数据丢失。 |
| **回环（loopback）** | 把 TX 和 RX 直接短接，发出去的东西原样收回来。 |

> 📌 **注解 · 为什么用 BufferedUart 而不是普通 Uart？**  
> 普通 UART 每读一个字节都要 `await`，CPU 参与多。BufferedUart 让中断在后台把字节收进/送出环形缓冲区，主程序只在缓冲区层面操作。USB↔UART 这种持续搬运的场景更适合它。

### 3.3 USB

| 名词 | 直白解释 |
| --- | --- |
| **USB 主机 / 设备** | 电脑是主机，DM-MC02 是设备。 |
| **端点（endpoint）** | USB 上的单向数据通道。IN = 设备→主机，OUT = 主机→设备。 |
| **Bulk 端点** | 大批量、可重传、不保证带宽的端点；CMSIS-DAP v2 和 CDC 数据都用它。 |
| **包（packet）** | USB 一次传输的数据块。全速 Bulk 端点最大 64 字节。 |
| **描述符（descriptor）** | 设备“自我介绍”的结构化数据：设备描述符、配置描述符、接口描述符、端点描述符、字符串描述符。 |
| **枚举（enumeration）** | 电脑插上设备后，问它要描述符、分配地址、选择配置的过程。 |
| **VID / PID** | Vendor ID / Product ID。本模板用 `0xC0DE:0xCAFC`，是课堂测试用的未分配 ID。 |
| **复合设备（composite）** | 一个 USB 设备里包含多个功能（这里是两个串口 + 一个调试接口）。 |
| **CDC ACM** | USB 虚拟串口类。 |
| **WinUSB** | Windows 上让用户态程序直接访问 USB 设备的驱动。 |
| **MS OS 描述符** | 微软扩展描述符，告诉 Windows 自动给这个接口绑定 WinUSB。 |
| **DTR** | Data Terminal Ready，串口的一个控制信号。电脑打开串口时通常拉高，固件用它判断“主机已连接”。 |
| **`/dev/cu.*`** | macOS 上“主动发起连接”的串口设备名；`/dev/tty.*` 是“等待连接”的版本。测试脚本用 `cu` 更合适。 |

> 📌 **注解 · 为什么插上板子会多出两个串口？**  
> 固件里创建了两个 `CdcAcmClass`，每个在系统里就是一个串口。它们共用同一个 USB 设备，但各自有独立的端点和状态。

### 3.4 CMSIS-DAP 与 SWD

| 名词 | 直白解释 |
| --- | --- |
| **CMSIS-DAP** | Arm 定义的调试探针 USB 协议。probe-rs、OpenOCD 都懂它。 |
| **CMSIS-DAP v2** | 用 Bulk 端点传输的版本；v1 用 HID。 |
| **DAP_Info** | CMSIS-DAP 的“查信息”命令，例如版本、能力、包大小。 |
| **DAP_Connect** | CMSIS-DAP 的“连接目标”命令。本模板返回 0，表示没有可连接的目标。 |
| **SWD** | Serial Wire Debug，两根线的调试接口：SWCLK + SWDIO。 |
| **SWCLK** | SWD 时钟，由主机产生。本作业用 PE9 模拟。 |
| **SWDIO** | SWD 双向数据线。本作业用 PE13 模拟。 |
| **DP** | Debug Port，调试端口的顶层。DPIDR、CTRL/STAT 等寄存器在这里。 |
| **AP** | Access Port，访问端口。通过 AP 可以读内存、外设寄存器。本作业只读 DP。 |
| **DPIDR** | Debug Port Identification Register，调试端口的“身份证”。Cortex-M7 常见值为 `0x6BA02477`。 |
| **ADIv5** | Arm Debug Interface v5，SWD 的协议规范。 |
| **Line reset** | SWDIO 连续 ≥50 个时钟为高，让总线回到已知状态。 |
| **JTAG-to-SWD** | 发送 16 位 `0xE79E`，把同时支持 JTAG 和 SWD 的调试口切到 SWD 模式。 |
| **Turnaround** | 双方交换驱动权时插入的空时钟，避免同时驱动。 |
| **ACK** | 目标回给主机的 3 位应答：`001` OK、`010` WAIT、`100` FAULT。 |
| **奇偶校验（parity）** | 数据里 1 的个数 + 校验位应为偶数；用来发现传输错误。 |
| **LSB first** | 最低位先发/先收。SWD 的请求、数据都是这个顺序。 |

> 📌 **注解 · DPIDR 为什么适合做测试？**  
> 读 DPIDR 不需要访问芯片内存，也不会改变目标状态，是最安全的一次“握手”。读到了，就说明时钟、数据、方向切换、ACK、校验整条链路都通了。

### 3.5 Rust 与 embassy

| 名词 | 直白解释 |
| --- | --- |
| **`no_std`** | 不用 Rust 标准库（没有操作系统、文件、`println!`、堆分配）。 |
| **`panic-halt`** | panic 后原地死循环；没有日志输出。 |
| **async / future** | 可以暂停和恢复的任务。`await` 表示“我先让出 CPU，等条件满足再回来”。 |
| **executor（执行器）** | 负责轮询 future 的调度器。embassy 的是单线程、协作式。 |
| **`join4`** | 四个 future 一起跑，等所有都结束；这里四个都是死循环，所以永不返回。 |
| **`select`** | 两个 future 一起跑，任意一个结束就返回。 |
| **`StaticCell`** | 给 `static` 变量做一次性初始化的容器；`init()` 返回 `&'static mut T`。 |
| **`Peri<'static, T>`** | 外设“使用许可证”。move 给别人后，编译期就不能再用同一个外设。 |
| **`bind_interrupts!`** | 把中断号和 embassy 的中断处理类型在编译期配对。 |
| **trait** | Rust 的接口约束。例如 `RxPin<UART7>` 表示“这个引脚能当 UART7 的 RX”。 |
| **`Option<T>`** | 可能有、可能没有。`Some(x)` 或 `None`。 |
| **`Result<T, E>`** | 成功 `Ok(x)` 或失败 `Err(e)`。 |
| **heapless** | 固定容量、不依赖堆的数据结构，例如 `heapless::String<63>`。 |
| **临界区（critical section）** | 短暂关中断，保证一段代码不被打断。 |

> 📌 **注解 · 一个执行器怎么同时跑四件事？**  
> 每件事都是一个 future。执行器轮流 poll 它们；某个 future 在 `await` 处返回 `Pending`，执行器就去 poll 下一个。外设中断发生时，会唤醒对应的 future，执行器再回来 poll 它。没有线程切换，也没有抢占。

### 3.6 工具链

| 名词 | 直白解释 |
| --- | --- |
| **probe-rs** | Rust 写的调试/烧录工具，支持 CMSIS-DAP、J-Link、ST-Link 等探针。 |
| **runner** | `.cargo/config.toml` 里的配置：`cargo run` 编译完后，用哪个命令运行产物。这里指向 `probe-rs run`。 |
| **RTT** | Real-Time Transfer，通过 SWD 输出日志的通道。本固件没用 RTT，所以烧录后终端没有输出。 |
| **ELF** | 可执行文件格式，包含代码、数据、符号和内存布局。probe-rs 烧的是 ELF 里的可加载段。 |
| **target triple** | 编译目标，例如 `thumbv7em-none-eabihf` = ARM Thumb、无操作系统、硬浮点 ABI。 |
| **`Cargo.lock`** | 锁定所有依赖的精确版本。 |
| **`--locked`** | 严格按 `Cargo.lock` 的版本构建，不允许自动升级。 |
| **链接脚本 / `memory.x`** | 告诉链接器代码和数据放到哪些地址。 |
| **OpenOCD** | 另一套开源调试工具，也支持 CMSIS-DAP。 |
| **DFU** | ST 系统 Bootloader 的 USB 固件升级模式。 |
| **venv** | Python 虚拟环境，隔离项目依赖。 |
| **pyserial** | Python 的串口库。 |

### 3.7 CAN（选做）

| 名词 | 直白解释 |
| --- | --- |
| **CAN** | 控制器局域网，机器人里常用的差分总线。 |
| **Classical CAN** | 经典 CAN，最多 8 字节数据。 |
| **CAN FD** | 灵活数据速率 CAN，数据可更长、速率可切换。本固件只配了 Classical。 |
| **收发器（transceiver）** | MCU 的 CAN 控制器输出逻辑电平，收发器把它变成 CAN_H/CAN_L 差分信号。 |
| **标准 ID** | 11 位标识符。 |
| **位时间 / tq** | 一个 CAN 位被分成若干时间份额（tq）。 |
| **采样点** | 在一个位里采样电平的位置，通常 75% 左右。 |
| **SJW** | Synchronization Jump Width，允许重新同步的范围。 |
| **终端电阻** | CAN 总线两端通常各接 120 Ω，抑制反射。 |

---

## 4. 硬件准备与接线

### 4.1 清单

- DM-MC02 板 ×1（同板自测足够）
- USB-C **数据线**（不是只充电的线）
- 助教提供的外置调试探针
- 杜邦线：
  - 烧录 3 根：探针 SWCLK / SWDIO / GND → P1
  - SWD 自测 2 根：P11 信号脚 → P1.3 / P1.4
  - UART 回环 1 根：P13.3 ↔ P13.4

### 4.2 板上接口位置

按 `考核指导` 的官方板卡位置图，把顶部的大电源插座作为方向基准：

- **P1（SWD）**：左下边缘
- **P11（四路 PWM 排针）**：右下角
- **P13（UART7）**：右侧 USB-C 上方

> ⚠️ **注意 · 看插座方向**  
> 从插座正面看和从线束背面看，左右是反的。按卡扣、开口和 1 脚标记核对。P1 的 1 脚是 3.3 V，接反可能把电源和信号短在一起。

### 4.3 P1（SWD 调试口）

| P1 脚号 | 信号 | 说明 |
| --- | --- | --- |
| 1 | 3.3 V | 电源 |
| 2 | GND | 地 |
| 3 | SWCLK | 接 MCU 的 PA14 |
| 4 | SWDIO | 接 MCU 的 PA13 |

烧录时：外置探针的 SWCLK → P1.3，SWDIO → P1.4，GND → P1.2。板子由 USB-C 供电。

> 📌 **注解 · VTREF**  
> 有些探针需要一根“目标电压参考”线来判断目标是不是上电、电平是多少。如果你的探针标了 VTref/VREF，按探针说明接 P1.1（3.3 V）。不要用探针给整块板供电。

### 4.4 P11（四路 PWM 排针）

原理图第 2 页左下：

| 通道 | 板上信号 | MCU 引脚 | 用途 |
| --- | --- | --- | --- |
| 第 1 路 | TIM2_CH1 | PA0 | 可作 SWCLK/SWDIO |
| 第 2 路 | TIM2_CH3 | PA2 | 可作 SWCLK/SWDIO |
| 第 3 路 | TIM1_CH1 | **PE9** | 当前代码选作 SWCLK |
| 第 4 路 | TIM1_CH3 | **PE13** | 当前代码选作 SWDIO |
| — | VCC_5V | — | ⚠️ 不要接 |
| — | GND | — | ⚠️ 不要接 |

> ⚠️ **注意 · P11 的“第几路”不是“第几脚”**  
> P11 是带 5 V 和 GND 的舵机排针。接线时认准**信号脚**，不要接到电源脚。用 DuPont 线之前先用万用表或丝印确认。

### 4.5 P13（UART7）

原理图第 2 页左上，4 针 A1257 插座：

| P13 脚号 | 信号 | MCU 引脚 |
| --- | --- | --- |
| 1 | VCC_5V | — |
| 2 | GND | — |
| 3 | UART7_RX | **PE7** |
| 4 | UART7_TX | **PE8** |

> ⚠️ **注意 · 脚号以四针原理图为准**  
> 说明书第 8 节的 UART7 表格和同一页的 4 针原理图有出入。`考核指导` 明确要求按四针原理图填写：1=5V、2=GND、3=RX、4=TX。回环只短接 3 和 4，绝不能碰 1 和 2。

### 4.6 其他串口（只作备选）

| 接口 | RX | TX | 位置 |
| --- | --- | --- | --- |
| UART7（推荐） | PE7 | PE8 | P13.3 / P13.4 |
| UART8 | PE0 | PE1 | P16.15 / P16.17（需要 P16 转接板） |
| UART10 | PE2 | PE3 | P15.3 / P15.4 |
| USART1 | PA10 | PA9 | P10 |

本作业用 UART7 即可，不要同时配置多路。

### 4.7 三种接线场景

#### 场景 A：外置探针烧录

```text
电脑 ── USB ── 外置探针             DM-MC02
                   GND ────────── P1.2
                 SWCLK ────────── P1.3
                 SWDIO ────────── P1.4
电脑 ── USB-C 数据线 ───────────── DM-MC02 USB-C
```

⚠️ 烧录前拔掉 P11 → P1 的所有自测跳线。

#### 场景 B：同板自测

```text
同一块 DM-MC02

P11 第 3 路信号脚（PE9）  ────── P1.3（SWCLK）
P11 第 4 路信号脚（PE13） ────── P1.4（SWDIO）

P13.4（TX，PE8） ────── P13.3（RX，PE7）
```

同板内部已经共地，只接两根信号线；不要接 3.3 V 或 5 V。

#### 场景 C：两板测试（可选）

```text
板 A（跑固件，当探针）        板 B（目标）
P11 信号脚（PE9） ────────── B 的 SWCLK
P11 信号脚（PE13）────────── B 的 SWDIO
GND ──────────────────────── GND

两块板各自供电。
```

两板测试必须接 GND，因为两块板的“地”不是同一个。

> ⚠️ **注意 · 绝对不能选 PA13/PA14 当 SWD GPIO**  
> PA13/PA14 是 P1 调试口自己的 SWDIO/SWCLK。你把它们当普通 GPIO 用，会占掉烧录/调试通道，后面 probe-rs 就连不上了。

---

## 5. Step 0：检查环境

### 5.1 做什么

打开终端，进入工程目录：

```sh
cd /Users/fangchuran/RoboMaster/硬件/培训/Embedded-test
```

检查工具：

```sh
git --version
rustup --version
cargo --version
probe-rs --version
```

检查 Python 环境：

```sh
source .venv/bin/activate
python3 -c "import serial; print(serial.__version__)"
```

### 5.2 预期

```text
git version 2.x
rustup 1.29.x
cargo 1.99.x
probe-rs 0.32.0
3.5
```

### 5.3 如果缺东西

| 缺什么 | 命令 |
| --- | --- |
| probe-rs | `cargo install probe-rs-tools --locked`，然后重开终端 |
| `.venv` | `python3 -m venv .venv && source .venv/bin/activate` |
| pyserial | `python3 -m pip install pyserial` |

> 📌 **注解 · 为什么用 `.venv`？**  
> 系统的 Python 可能被其他项目依赖。虚拟环境让 `pyserial` 只装在工程目录的 `.venv` 里，不污染系统。`.gitignore` 已经把 `.venv` 排除了。

> 📌 **注解 · macOS 需要 udev 规则吗？**  
> 不需要。udev 是 Linux 的机制；macOS 直接通过 USB 访问，通常不需要额外驱动或 `sudo`。

---

## 6. Step 1：编译工程

### 6.1 做什么

```sh
cargo build --locked
```

### 6.2 为什么

- 确认工具链、目标架构、依赖版本都没问题。
- 生成可烧录的 ELF。
- 先确认当前代码能编译，再去烧录和实测，避免把“代码错误”和“硬件问题”混在一起。

### 6.3 预期

```text
   Compiling ...
    Finished `dev` profile [optimized + debuginfo] target(s) in ...s
```

产物：

```text
target/thumbv7em-none-eabihf/debug/dm-mc02-cmsis-dap
```

> 📌 **注解 · `thumbv7em-none-eabihf`**  
> - `thumbv7em`：ARM Thumb 指令集，Cortex-M4/M7 级别。
> - `none`：没有操作系统。
> - `eabihf`：Embedded ABI，硬浮点。
>
> 这个三元组决定了编译器生成什么样的机器码。

> 📌 **注解 · `--locked`**  
> 严格使用 `Cargo.lock` 里的版本。作业要求锁定依赖，避免你本机编译用新版本、助教机器用旧版本导致行为不同。

> 📌 **注解 · 为什么产物叫 ELF 但没扩展名？**  
> ELF 是文件格式，不是扩展名。probe-rs 通过文件头识别它，烧录时取其中的可加载段写入 Flash。

### 6.4 卡住了

| 现象 | 先检查 |
| --- | --- |
| 找不到 `cargo` | 终端 PATH；重开终端 |
| 工具链下载失败 | 网络；不要删 `rust-toolchain.toml` 的版本号 |
| 依赖下载失败 | 网络/代理；保留报错找助教 |
| 编译错误 | 看 Step 11 的错误表 |

> ✅ **检查点**：看到 `Finished` 再往下走。

---

## 7. Step 2：可选——先烧原始模板

### 7.1 什么时候做

如果你**还没有烧过任何固件**，建议先用原始模板跑一遍，把「硬件/探针/USB」问题和「代码」问题分开。  
如果你已经烧过、或者不想来回改代码，可以跳过，直接烧当前版本。

> 当前仓库的 `wiring.rs` 已经改过了。想烧原始模板，需要先 `git stash` 暂存改动，烧完再 `git stash pop`。不想折腾就跳过。

### 7.2 做什么

1. 断电接线（场景 A）。
2. 确认探针被识别：

```sh
probe-rs list
```

3. 烧录并运行：

```sh
cargo run --locked
```

4. 看到烧录完成后按 `Ctrl+C`。

### 7.3 预期

- `probe-rs list` 看到外置探针。
- 烧录后终端停住、没有日志（因为固件没有 RTT 输出）。
- `Ctrl+C` 后板子继续运行。
- 重新插拔 USB-C 后，`probe-rs list` 多出 `DM-MC02 CMSIS-DAP Demo`。
- `python3 -m serial.tools.list_ports` 看到两个新串口。
- 对控制口 `ping` 返回 `OK DM-MC02`。
- 对控制口 `swd` 返回 `ERR SWD_NOT_CONFIGURED`（因为模板里 `swd = None`）。

> 📌 **注解 · `cargo run` 为什么不是在本机运行程序？**  
> `.cargo/config.toml` 把 runner 设成了：
>
> ```text
> probe-rs run --chip STM32H723VGTx --protocol swd ...
> ```
>
> 所以 `cargo run` = 先编译，再调用 probe-rs 把 ELF 烧进 STM32，然后复位运行。它不会在 macOS 上执行这个 ARM 程序。

> 📌 **注解 · 为什么终端没有输出？**  
> 本固件没有启用 RTT（Real-Time Transfer）日志。probe-rs 会保持一个会话等待日志，所以看起来“卡住”。这是正常的。

> 📌 **注解 · 烧录时探针被占用**  
> `probe-rs run` 运行期间会占着探针；这时另一个终端再跑 `probe-rs list` 或 `cargo run` 可能失败。按 `Ctrl+C` 退出后才会释放。

---

## 8. Step 3：确认引脚选择

### 8.1 作业对引脚的要求

README 原文：

> 从板卡上选两个可接线的 3.3 V GPIO。SWCLK 用初始电平为低的 `Output`，SWDIO 用可切换方向的 `Flex`。记录 MCU 引脚和接插件脚号。可以从外露的 PWM 信号脚里选。

所以要求只有三条：

1. 两个**外露、可接线**的 3.3 V GPIO。
2. SWCLK 用 `Output`，初始低。
3. SWDIO 用 `Flex`，能切换方向。

### 8.2 当前代码的选择

你现在的 `wiring.rs` 用的是：

| 信号 | MCU 引脚 | P11 通道 | 回接目标 |
| --- | --- | --- | --- |
| SWCLK | **PE9** | 第 3 路（TIM1_CH1） | P1.3 |
| SWDIO | **PE13** | 第 4 路（TIM1_CH3） | P1.4 |

UART：

| 信号 | MCU 引脚 | P13 脚号 |
| --- | --- | --- |
| UART TX | **PE8** | P13.4（UART7_TX） |
| UART RX | **PE7** | P13.3（UART7_RX） |

### 8.3 和考核指导的差异

`考核指导` 的示例用的是 **PA0 / PA2**（P11 第 1、2 路）。这和你现在的 PE9/PE13 都符合 README 的要求，区别只是选哪两路 PWM 信号。

建议：**保持 PE9/PE13 不变**。理由：

- 代码已经写好并编译通过；
- P11 第 3、4 路信号脚相邻，接线方便；
- 不占用 PA13/PA14；
- 不和 UART7 冲突。

> ⚠️ **注意 · 引脚表必须和代码一致**  
> 如果你后面改用 PA0/PA2，要同时改 `wiring.rs`、跳线位置和提交的引脚表。三处必须一致，否则验收会失败。

### 8.4 冲突检查

| 引脚 | 本固件里的用途 | 是否冲突 |
| --- | --- | --- |
| PA11 / PA12 | USB D− / D+ | 不选 |
| PA13 / PA14 | P1 调试口 SWDIO / SWCLK | 不选 |
| PA9 / PA10 | USART1 | 不选 |
| PE0 / PE1 | UART8 | 不选 |
| PE2 / PE3 | UART10 | 不选 |
| PE7 / PE8 | UART7（本作业用） | 与 PE9/PE13 不冲突 |
| PE9 / PE13 | SWD GPIO（本作业用） | 与 UART7 不冲突 |
| PB15 | 板载蜂鸣器 | 不选 |
| PC4 | VBAT 电压采集 | 不选 |

---

## 9. Step 4：实现 TODO 1（SWD GPIO）

### 9.1 做什么

你的仓库里**已经写了**这一段。如果你以后从头再做一遍，或者换了一台机器，做法是：

打开 `src/wiring.rs`，把原来的：

```rust
let swd = None;
```

改成：

```rust
let swd = Some(Swd::new(
    Output::new(p.PE9, Level::Low, Speed::Low), // SWCLK，初始低
    Flex::new(p.PE13),                          // SWDIO，可切换方向
));
```

并在文件顶部补导入：

```rust
use embassy_stm32::gpio::{Flex, Level, Output, Speed};
```

已经写好的话，直接看 9.2 的逐行解释和 9.4 的编译检查。

### 9.2 逐行解释

```rust
Some(...)
```

`configure()` 的返回类型是 `Option<Swd>`。`Some` 表示“已配置”，这样 `protocol.rs` 里的 `SWD ID` 命令才会走真正读 DPIDR 的分支；如果是 `None`，会回 `ERR SWD_NOT_CONFIGURED`。

```rust
Output::new(p.PE9, Level::Low, Speed::Low)
```

- `p.PE9`：把 PE9 这个外设“许可证”move 给 `Output`。
- `Level::Low`：初始电平为低。SWCLK 空闲时是低电平，避免上电时产生一个假的上升沿。
- `Speed::Low`：输出摆率低。信号是 3.3 V、线不长，低速更稳、干扰更小。

```rust
Flex::new(p.PE13)
```

- `Flex` 可以在输入和输出之间切换，SWDIO 需要双向。
- `Flex::new` 初始是“断开”状态；`Swd::new` 内部会调用 `data.set_as_input(Pull::Up)`，把它设成输入上拉。

```rust
Swd::new(clock, data)
```

参数顺序是**时钟在前、数据在后**。`swd.rs` 里：

```rust
pub fn new(mut clock: Output<'static>, mut data: Flex<'static>) -> Self {
    clock.set_low();
    data.set_as_input(Pull::Up);
    Self { clock, data }
}
```

### 9.3 为什么不能用 `Output` 驱动 SWDIO

`Output` 会一直主动驱动线。SWD 读操作里，目标要回 ACK 和数据，如果主机还在驱动，就会“两个输出打架”。`Flex` 允许主机在 turnaround 时松手。

### 9.4 编译检查

```sh
cargo build --locked
```

如果报 `Peri`/`Output` 类型不匹配，先确认 `p.PE9` 是 `Peri<'static, PE9>`，以及 `Swd::new` 需要的是 `Output<'static>` 和 `Flex<'static>`。正常写法不需要手动标注生命周期。

> ✅ **检查点**：TODO 1 单独能编译，再写 TODO 2。

---

## 10. Step 5：实现 TODO 2（Buffered UART）

### 10.1 做什么

你的仓库里**已经写了**这一段。下面是对照用的完整写法；如果以后重做，按这个顺序补。

#### 第一步：文件级绑定中断

在 `wiring.rs` 顶部、`configure()` 外面加：

```rust
use embassy_stm32::{bind_interrupts, peripherals, usart};

bind_interrupts!(struct UartIrqs {
    UART7 => usart::BufferedInterruptHandler<peripherals::UART7>;
});
```

#### 第二步：构造 UART

把：

```rust
let uart = None;
```

改成：

```rust
static UART_TX: StaticCell<[u8; 256]> = StaticCell::new();
static UART_RX: StaticCell<[u8; 256]> = StaticCell::new();

let mut uart_config = usart::Config::default(); // 默认 115200 8N1
uart_config.baudrate = 115_200;

let uart = Some(
    BufferedUart::new(
        p.UART7,
        p.PE7,                        // RX 在前
        p.PE8,                        // TX 在后
        UART_TX.init([0; 256]),       // TX 缓冲在前
        UART_RX.init([0; 256]),       // RX 缓冲在后
        UartIrqs,
        uart_config,
    )
    .expect("UART7 115200 8N1"),
);
```

### 10.2 参数顺序（重点）

```text
BufferedUart::new(
    外设,        // p.UART7
    RX 引脚,     // p.PE7
    TX 引脚,     // p.PE8
    TX 缓冲区,   // UART_TX.init(...)
    RX 缓冲区,   // UART_RX.init(...)
    中断绑定,    // UartIrqs
    配置,        // uart_config
)
```

两个反直觉点：

1. **RX 引脚在 TX 引脚前面。**
2. **TX 缓冲区在 RX 缓冲区前面。**

> 📌 **注解 · 为什么缓冲区必须是 `'static`？**  
> `BufferedUart<'static>` 会活得比 `configure()` 函数长。如果缓冲区是函数里的局部数组，函数返回后就被销毁了，UART 中断会访问野指针。`StaticCell` 把数组放在静态存储区，只初始化一次，返回 `&'static mut`。

> 📌 **注解 · `bind_interrupts!` 在做什么？**  
> 它把中断向量 `UART7` 和 `usart::BufferedInterruptHandler<peripherals::UART7>` 在编译期配对。运行时 UART7 收到字节、发送空等事件触发中断，就进入 embassy 的处理函数，由它把数据搬进/搬出你的静态缓冲区。

### 10.3 为什么用 `expect`

`BufferedUart::new` 返回 `Result<Self, ConfigError>`。用 `.expect(...)`：

- 成功时得到 `BufferedUart`；
- 配置非法时直接 panic，现场停机，避免“UART 悄悄没工作”的假象。

如果作业要求不 panic，也可以 `.ok()`，但调试时更建议 `expect`。

### 10.4 编译检查

```sh
cargo build --locked
```

常见错误：

| 报错 | 原因 |
| --- | --- |
| `PE7 does not implement RxPin<UART7>` | RX/TX 写反或外设写错 |
| `PE8 does not implement TxPin<UART7>` | 同上 |
| `expected BufferedUart<'static>` | 缓冲区用了局部数组 |
| `cannot find type UART7` | 没写 `peripherals::UART7` |
| `Binding` 类型不匹配 | 中断绑定用了非 buffered 的 handler |

> ✅ **检查点**：完整 `cargo build --locked` 通过。

---

## 11. Step 6：完整编译

### 11.1 做什么

```sh
cargo build --locked
```

### 11.2 预期

```text
    Finished `dev` profile [optimized + debuginfo] target(s) in ...s
```

当前这份 `wiring.rs` 已经在一份临时副本里编译通过，所以你正常应该看到 `Finished`。

### 11.3 可选的格式化和静态检查

```sh
cargo fmt           # 格式化代码
cargo clippy --locked
```

> 当前 `wiring.rs` 的缩进有点乱，不影响编译。想整洁一点可以 `cargo fmt`；不想动就保持现状。

### 11.4 常见编译错误怎么读

Rust 的错误信息一般会指出：

```text
error[E0277]: the trait bound `PE7: RxPin<UART7>` is not satisfied
  --> src/wiring.rs:...
```

读法：

1. **先看 trait bound**：谁没有实现哪个 trait。
2. **再看 `-->` 指向哪一行**。
3. 如果是引脚 trait，优先怀疑 RX/TX 顺序或外设型号。
4. 如果是生命周期，优先怀疑缓冲区不是 `'static`。

---

## 12. Step 7：烧录新版

### 12.1 烧录前检查

- [ ] P11 → P1 的自测跳线**已拔掉**
- [ ] 外置探针接 P1.2 / P1.3 / P1.4，方向正确
- [ ] DM-MC02 通过 USB-C 数据线供电
- [ ] 没有其他 `probe-rs` / `cargo run` 会话占着探针

### 12.2 做什么

```sh
probe-rs list
cargo run --locked
```

### 12.3 预期

`probe-rs list` 看到外置探针。  
`cargo run` 编译、烧录、复位。**终端停住不输出是正常的**。烧录完成后按 `Ctrl+C`。

### 12.4 如果 probe-rs 让你选探针

列表里可能同时有：

- 外置探针
- `DM-MC02 CMSIS-DAP Demo`（如果板子已经烧过固件）

**选外置探针**。演示接口不能烧录。

也可以直接用 VID:PID 指定：

```sh
probe-rs run --probe VID:PID --chip STM32H723VGTx --protocol swd \
  target/thumbv7em-none-eabihf/debug/dm-mc02-cmsis-dap
```

### 12.5 烧录后

1. 按 `Ctrl+C` 退出 probe-rs（释放探针）。
2. **拔掉外置探针**（避免它继续驱动 P1.3/P1.4）。
3. 断电，再重新插 USB-C 给板子上电。

> 📌 **注解 · 为什么要拔外置探针？**  
> 自测时，PE9/PE13 会驱动 P1.3/P1.4。如果外置探针还接在上面，它的输出也会驱动同样的线，两边打架，可能读不到 DPIDR，甚至损坏探针。

> 📌 **注解 · 为什么断电重上电？**  
> 烧录后芯片已经被 probe-rs 复位并运行；重新上电可以确保它从 Flash 正常启动，外设状态干净。

---

## 13. Step 8：上电与接测试跳线

### 13.1 SWD 回接

```text
P11 第 3 路信号脚（PE9）  ────── P1.3（SWCLK）
P11 第 4 路信号脚（PE13） ────── P1.4（SWDIO）
```

同板只接两根信号线。板内已经共地，不要接 P1.1 的 3.3 V。

### 13.2 UART 回环

```text
P13.4（TX，PE8） ────── P13.3（RX，PE7）
```

只短接 3 和 4；P13.1 是 5 V，P13.2 是 GND。

### 13.3 检查

- [ ] 两根 SWD 跳线接在信号脚，不是 5V/GND
- [ ] UART 跳线接在 P13.3 / P13.4
- [ ] 外置探针已拔掉
- [ ] USB-C 已插回电脑

> ⚠️ **注意 · 不要带电插拔跳线**  
> 尽量断电接线，接好再上电。带电插拔容易把信号脚瞬间短到电源或地。

---

## 14. Step 9：USB 枚举与端口识别

### 14.1 做什么

```sh
probe-rs list
python3 -m serial.tools.list_ports -v
```

### 14.2 预期

`probe-rs list`：

```text
DM-MC02 CMSIS-DAP Demo
```

串口列表：两个新串口，例如：

```text
/dev/cu.usbmodem12301
/dev/cu.usbmodem12303
```

> 📌 **注解 · 为什么有两个串口？**  
> 固件里创建了两个 CDC ACM 类：一个给控制命令，一个给 UART 透传。它们在系统里长得一模一样，只能靠行为区分。

### 14.3 找控制口

分别对两个口发 `PING`：

```sh
python3 host.py /dev/cu.usbmodem12301 ping
python3 host.py /dev/cu.usbmodem12303 ping
```

- 回 `OK DM-MC02` 的是**控制口**。
- 没反应或回显 `PING` 的是 **UART 口**。

记下来：

```sh
export CONTROL=/dev/cu.usbmodem12301
export UART=/dev/cu.usbmodem12303
```

### 14.4 输出注解

```text
OK DM-MC02
│  │
│  └─ 固定产品标识
└─ 命令执行成功
```

如果没有回复：

```text
No response. Check that this is the control interface.
```

说明你选的不是控制口，或者板子没在跑固件。

> 📌 **注解 · `wait_connection` 和 DTR**  
> 控制口在 `wait_connection().await` 处等电脑“打开串口”。`host.py` 用 `serial.Serial` 打开时会拉高 DTR，固件才认为主机已连接。如果你用别的串口工具，确认它拉高了 DTR。

> 📌 **注解 · 端口被占用**  
> 如果报 `Resource busy`，检查是否有 `screen`、Arduino IDE、其他 `host.py` 进程占着这个口。macOS 上 `lsof /dev/cu.usbmodem...` 可以查。

---

## 15. Step 10：SWD 验收

### 15.1 做什么

```sh
python3 host.py $CONTROL swd
python3 host.py $CONTROL swd
python3 host.py $CONTROL ping
```

### 15.2 预期

```text
OK DPIDR 6BA02477
OK DPIDR 6BA02477
OK DM-MC02
```

> 具体值以实测为准；Cortex-M7 常见 `0x6BA02477`。关键是**两次完全一致**。

### 15.3 输出注解

```text
OK DPIDR 6BA02477
│  │     │
│  │     └─ 8 位十六进制 DPIDR
│  └─ 读的是 DPIDR 寄存器
└─ 命令成功
```

### 15.4 固件内部发生了什么

```text
host.py 发 "SWD ID\n"
 → USB OUT
 → control_port 攒成一行
 → protocol::execute
 → swd.read_dpidr()
     → PE9 打 56 个时钟（line reset）
     → PE13 发 0xE79E（JTAG→SWD）
     → 再 line reset
     → 发 0xA5（读 DP 寄存器 0）
     → PE13 切成输入
     → 读 3 位 ACK
     → 读 32 位数据 + 1 位校验
     → 收尾
 → 成功："OK DPIDR XXXXXXXX\n"
 → 失败："ERR SWD\n"
```

### 15.5 错误对照

| 回复 | 含义 | 先检查 |
| --- | --- | --- |
| `ERR SWD_NOT_CONFIGURED` | `swd` 还是 `None` | 是否烧了新固件；`wiring.rs` 是否 `Some(...)` |
| `ERR SWD` | SWD 链路失败 | 跳线、P1 方向、外置探针是否拔掉、PE9/PE13 是否接反 |
| 两次 DPIDR 不同 | 链路不稳 | 杜邦线接触、线太长、目标供电 |
| 读不到但 `ping` 正常 | MCU 没死，只是 SWD 不通 | 回到接线和引脚选择 |

> 📌 **注解 · 为什么读两次？**  
> 一次成功可能是运气；两次一致说明时钟/数据/方向切换稳定。验收表明确要求两次 DPIDR 相同。

> 📌 **注解 · 为什么最后再 `ping`？**  
> 读 DPIDR 是同步阻塞的位操作，如果时序或方向切换有问题，可能让执行器卡住。再 `ping` 一次，确认 MCU 还活着。

### 15.6 可选实验：确认引脚没有接反

如果你想在实测记录里写“用实验确认了 PE9/PE13 的分配”，可以：

1. 正常接 P11 第 3 路 → P1.3、第 4 路 → P1.4，记录 `OK DPIDR ...`。
2. 断电，把两根线对调（第 3 路 → P1.4、第 4 路 → P1.3）。
3. 上电，执行 `swd`，预期 `ERR SWD`。
4. 断电换回，再测，预期恢复 `OK DPIDR ...`。

这三次结果能证明“时钟/数据方向不是碰巧对的”。

---

## 16. Step 11：UART 回环验收

### 16.1 做什么

确认 P13.4（TX）和 P13.3（RX）已经短接，然后：

```sh
python3 host.py $UART uart-loopback
```

### 16.2 预期

```text
OK UART loopback
```

### 16.3 固件内部发生了什么

```text
host.py 写 b"HerKules UART\n"
 → USB OUT
 → uart_port 的 from_host
 → BufferedUart TX → PE8（UART7_TX）
 → 跳线
 → PE7（UART7_RX）→ BufferedUart RX
 → uart_port 的 to_host
 → USB IN
 → host.py 读回 13 字节并逐字节比较
```

`host.py` 的期望值：

```python
expected = b"HerKules UART\n"
```

共 13 字节。

### 16.4 错误对照

| 现象 | 先检查 |
| --- | --- |
| 收到空 `b''` | 跳线没接、接错脚、UART 没配置 |
| 收到乱码 | 波特率/引脚 AF 不对，或 RX/TX 写反 |
| 回显 `PING` 之类 | 打开了控制口，不是 UART 口 |
| `Resource busy` | 端口被其他程序占用 |
| 超时 | P13.3/P13.4 没短接，或 UART7 中断没绑定 |

### 16.5 实测确认实验（推荐写进记录）

这是 `考核指导` 建议的做法，能证明数据确实经过了你选的物理引脚：

1. 接上 P13.3–P13.4 跳线，运行 `uart-loopback` → `OK UART loopback`。
2. 断电，拔掉跳线，上电，再运行 → 预期超时/收不到。
3. 断电，接回跳线，上电，再运行 → 预期恢复 `OK UART loopback`。

记录这三次输出，就能说明：

- 数据不是 USB 内部回环；
- 确实走了 P13.3 / P13.4；
- 也就是走了 PE7 / PE8 的 UART7。

> 📌 **注解 · 为什么这个实验有说服力？**  
> 如果固件只是把 USB 收到的数据原样写回 USB，不接跳线也会“回环成功”。拔掉跳线后失败、接回后成功，才能排除这种假回环。

---

## 17. Step 12：选做 CAN

### 17.1 前提

需要另一个 CAN 节点（第二块板、CAN 分析仪或电机驱动器），并且：

- 波特率 1 Mbit/s；
- Classical CAN（不是 FD）；
- 总线两端按需要接 120 Ω 终端电阻；
- CAN_H / CAN_L / GND 接对。

### 17.2 命令

```sh
# 发送标准帧 ID=0x123，数据 11 22 33 44
python3 host.py $CONTROL can-send 123 11223344

# 接收一帧（最多等 100 ms）
python3 host.py $CONTROL can-recv
```

### 17.3 预期

发送：

```text
OK QUEUED
```

接收：

```text
OK RX 123 11223344
```

空数据帧：

```text
OK RX 123 -
```

100 ms 内没收到：

```text
EMPTY
```

### 17.4 位时序

`board.rs` 里的配置：

```text
prescaler = 1
seg1      = 59
seg2      = 20
SJW       = 20
时钟      = 80 MHz
```

```text
波特率 = 80 MHz / (1 × (1 + 59 + 20)) = 1 Mbit/s
采样点 = (1 + 59) / (1 + 59 + 20) = 75%
```

### 17.5 不做怎么办

如果没做，就在实测记录里写：

```text
未测：CAN（缺少第二个 CAN 节点）
```

选做不影响必做项的验收。

---

## 18. Step 13：记录与提交

### 18.1 引脚表模板

按你当前的代码（PE9/PE13 + UART7）：

| 信号 | MCU 引脚 | 接插件脚号 | 同板回接到 P1 |
| --- | --- | --- | --- |
| SWCLK | PE9 | P11 第 3 路信号脚（TIM1_CH1） | P1.3 |
| SWDIO | PE13 | P11 第 4 路信号脚（TIM1_CH3） | P1.4 |
| UART TX | PE8 | P13.4（UART7_TX） | — |
| UART RX | PE7 | P13.3（UART7_RX） | — |

> 如果你改成 PA0/PA2，把前两行换成「P11 第 1 路信号脚（TIM2_CH1）」和「P11 第 2 路信号脚（TIM2_CH3）」。

### 18.2 实测记录模板

```text
日期：
板卡：DM-MC02（STM32H723VGT6）
固件 commit：
外置探针型号：
控制口：/dev/cu.usbmodem...
UART 口：/dev/cu.usbmodem...

一、USB 发现
$ probe-rs list
（粘贴实际输出）

二、控制口
$ python3 host.py $CONTROL ping
OK DM-MC02

三、SWD
$ python3 host.py $CONTROL swd
OK DPIDR 0x........
$ python3 host.py $CONTROL swd
OK DPIDR 0x........
$ python3 host.py $CONTROL ping
OK DM-MC02

四、UART 回环
$ python3 host.py $UART uart-loopback
OK UART loopback

五、实测确认的配置
接跳线：OK UART loopback
拔跳线：超时/收不到
接回：OK UART loopback
结论：数据确实经过 P13.3/P13.4，也就是 PE7/PE8。

六、未测项目
CAN（缺少第二个节点）
两板测试（未做）
```

### 18.3 提交命令

```sh
git status
git add src/wiring.rs
git commit -m "Complete SWD GPIO and UART7 wiring"
git push
```

> ⚠️ **注意 · 不要提交 `target/` 和 `.venv/`**  
> `.gitignore` 已经排除了它们。`Guide.md` 和 `考核指导/` 是新文件，是否提交由你决定；提交前先看 `git status`。

### 18.4 提交前检查清单

- [ ] `cargo build --locked` 通过
- [ ] `probe-rs list` 能看到 `DM-MC02 CMSIS-DAP Demo`
- [ ] `ping` 返回 `OK DM-MC02`
- [ ] 两次 `swd` 返回相同 DPIDR
- [ ] `swd` 后再 `ping` 仍正常
- [ ] `uart-loopback` 返回 `OK UART loopback`
- [ ] 引脚表和代码一致
- [ ] 实测记录包含实际端口名和实际输出

---

## 19. 故障排查总表

| 现象 | 可能原因 | 检查顺序 |
| --- | --- | --- |
| `cargo build` 失败 | 工具链/依赖/代码错误 | 先看第一个 `error`；对照 Step 11 |
| `probe-rs list` 没有探针 | USB 线、驱动、探针被占用 | 换线、换口、关掉其他 probe-rs |
| 找到探针但连不上芯片 | P1 方向、GND、供电 | 按场景 A 重新接线；确认板子上电 |
| 提示探针被占用 | 还有 `cargo run`/VS Code 调试器 | `Ctrl+C`，关掉调试会话 |
| USB 看不到演示探针和串口 | 板子没跑固件/停在 DFU/线是充电线 | 重新烧录；检查 P14/BOOT0；换数据线 |
| `ping` 无响应 | 选错串口/端口被占用 | 试另一个串口；`lsof` 查占用 |
| `ERR SWD_NOT_CONFIGURED` | 烧的是旧固件/`swd` 仍是 `None` | 重新编译烧录；检查 `wiring.rs` |
| `ERR SWD` | 跳线错/P1 接反/探针没拔 | 检查 PE9→P1.3、PE13→P1.4；拔探针 |
| 两次 DPIDR 不同 | 接触不良/线太长/干扰 | 短接线、重新插紧、降低线长 |
| UART 回环收空 | 跳线没接/接错/UART 没配置 | 短接 P13.3–P13.4；检查中断绑定 |
| UART 回环乱码 | 波特率/引脚顺序错 | 确认 115200 8N1；RX=PE7、TX=PE8 |
| 回环回显 `PING` | 打开了控制口 | 用另一个串口 |
| `Resource busy` | 端口被占用 | 关闭 `screen`/IDE/其他 host.py |
| CAN `EMPTY` | 总线上没有帧/速率不对 | 检查第二个节点、1 Mbit/s、终端电阻 |
| CAN `ERR CAN TX` | 总线无 ACK/收发器未供电 | 检查另一节点和 CAN_H/CAN_L |

---

## 20. 附录 A：`wiring.rs` 参考实现

下面是与当前仓库功能一致的格式化版本。你仓库里的代码缩进略乱，但编译已经通过。

```rust
//! Complete the two TODOs in this file.

use embassy_stm32::{Peripherals, can::Can, usart::BufferedUart};
use embassy_stm32::gpio::{Flex, Level, Output, Speed};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use static_cell::StaticCell;

use crate::swd::Swd;

bind_interrupts!(struct UartIrqs {
    UART7 => usart::BufferedInterruptHandler<peripherals::UART7>;
});

pub struct Ports {
    pub can: Can<'static>,
    pub uart: Option<BufferedUart<'static>>,
}

pub fn configure(p: Peripherals) -> (crate::board::Usb, Ports, Option<Swd>) {
    let usb = crate::board::usb(p.USB_OTG_HS, p.PA12, p.PA11);

    // TODO 1：SWCLK = PE9（P11 第 3 路信号脚），SWDIO = PE13（第 4 路）。
    let swd = Some(Swd::new(
        Output::new(p.PE9, Level::Low, Speed::Low),
        Flex::new(p.PE13),
    ));

    // TODO 2：UART7，RX = PE7（P13.3），TX = PE8（P13.4），115200 8N1。
    static UART_TX: StaticCell<[u8; 256]> = StaticCell::new();
    static UART_RX: StaticCell<[u8; 256]> = StaticCell::new();

    let mut uart_config = usart::Config::default();
    uart_config.baudrate = 115_200;

    let uart = Some(
        BufferedUart::new(
            p.UART7,
            p.PE7,
            p.PE8,
            UART_TX.init([0; 256]),
            UART_RX.init([0; 256]),
            UartIrqs,
            uart_config,
        )
        .expect("UART7 115200 8N1"),
    );

    let can = crate::board::can1(p.FDCAN1, p.PD0, p.PD1);
    (usb, Ports { can, uart }, swd)
}
```

> 如果你想换成考核指导里的 PA0/PA2，只改两行：
>
> ```rust
> Output::new(p.PA0, Level::Low, Speed::Low),
> Flex::new(p.PA2),
> ```
>
> 同时把跳线改到 P11 第 1、2 路，并更新引脚表。

---

## 21. 附录 B：源码导读

### 21.1 `main.rs`

```text
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let (driver, ports, swd) = wiring::configure(board::init());
    // 建 USB 复合设备：DAP + 两个 CDC
    // join4 同时跑：USB 协议栈、控制口、UART 口、DAP
}
```

- `board::init()` 配时钟，返回 `Peripherals`。
- `wiring::configure()` 把外设分给 USB、CAN、SWD、UART。
- `Builder` 拼描述符；`device.run()` 处理枚举和控制传输。
- `join4` 让四个 future 并发跑。

`control_port`：

```text
USB 包 → 逐字节攒行 → 遇到 \n → protocol::execute → 回包
```

`uart_port`：

```text
USB 读 → UART 写
UART 读 → USB 写
select 同时跑两个方向
```

### 21.2 `board.rs`

- `init()`：HSE 24 MHz、PLL2 给 FDCAN 80 MHz、HSI48 给 USB。
- `usb()`：`Driver::new_fs`，PA12/PA11，USB 中断绑定。
- `can1()`：FDCAN1，PD0/PD1，1 Mbit/s、75% 采样点、Classical CAN。

### 21.3 `protocol.rs`

- `Line`：96 字节行缓冲，`\n` 结束。
- `execute`：`PING`、`SWD ID`、`CAN TX`、`CAN RX`。
- `Reply = String<63>`：不占堆，保证一个 USB 包能装下。
- CAN 读写都带 100 ms 超时，避免控制口被卡死。

### 21.4 `dap_usb.rs`

- 建一个 class 0xFF 的 CMSIS-DAP v2 接口。
- 两个 Bulk 端点，64 字节。
- MS OS 描述符让 Windows 绑定 WinUSB。
- `discovery_reply` 只回答 `DAP_Info`，`DAP_Connect` 返回 0。
- 结论：它只用于 `probe-rs list` 发现，不能烧录。

### 21.5 `swd.rs`

- `clock()`：低电平期间采样，上升沿让目标采样。
- `write_bits` / `read_bits`：LSB first。
- `read_dpidr()`：
  1. line reset（56 个高电平）
  2. `0xE79E` JTAG→SWD
  3. 再 line reset
  4. `0xA5` 读 DP 寄存器 0
  5. 切输入、读 ACK、读 32 位数据 + 校验
  6. 收尾

### 21.6 `host.py`

- `serial.Serial(port, 115200, timeout=2)`
- `ping` → `PING\n`
- `swd` → `SWD ID\n`
- `can-send` → `CAN TX <id> <data>\n`
- `can-recv` → `CAN RX\n`
- `uart-loopback` → 发 `b"HerKules UART\n"`，读回比较

---

## 22. 附录 C：引脚速查

### 22.1 P1（SWD）

| 脚 | 信号 |
| --- | --- |
| 1 | 3.3 V |
| 2 | GND |
| 3 | SWCLK（PA14） |
| 4 | SWDIO（PA13） |

### 22.2 P11（PWM / 舵机）

| 通道 | 信号 | MCU |
| --- | --- | --- |
| 1 | TIM2_CH1 | PA0 |
| 2 | TIM2_CH3 | PA2 |
| 3 | TIM1_CH1 | PE9 |
| 4 | TIM1_CH3 | PE13 |
| — | VCC_5V | — |
| — | GND | — |

### 22.3 P13（UART7）

| 脚 | 信号 | MCU |
| --- | --- | --- |
| 1 | VCC_5V | — |
| 2 | GND | — |
| 3 | UART7_RX | PE7 |
| 4 | UART7_TX | PE8 |

### 22.4 P15（UART10）

| 脚 | 信号 | MCU |
| --- | --- | --- |
| 1 | VCC_5V | — |
| 2 | GND | — |
| 3 | USART10_RX | PE2 |
| 4 | USART10_TX | PE3 |

### 22.5 P16（扩展 IO，节选）

| 脚 | 信号 | MCU |
| --- | --- | --- |
| 9 | UART9_TX | PD15 |
| 11 | UART9_RX | PD14 |
| 13 | GPIO | PE14 |
| 15 | UART8_RX | PE0 |
| 17 | UART8_TX | PE1 |
| 19 | SPI3_MOSI | PC12 |
| 21 | SPI3_MISO | PC11 |
| 22 | IIC_SCL | PB8 |
| 23 | SPI3_CLK | PC10 |
| 24 | IIC_SDA | PB9 |

### 22.6 不要用的引脚

| 引脚 | 原因 |
| --- | --- |
| PA13 / PA14 | P1 调试口，烧录用 |
| PA11 / PA12 | USB D−/D+ |
| PB15 | 板载蜂鸣器 |
| PC4 | VBAT 电压采集 |
| PE0 / PE1 | UART8（除非你选 UART8） |
| PE2 / PE3 | UART10 |

---

## 23. 附录 D：命令速查卡

```sh
# 进入工程
cd /Users/fangchuran/RoboMaster/硬件/培训/Embedded-test

# 激活 Python 环境
source .venv/bin/activate

# 编译
cargo build --locked

# 查看探针
probe-rs list

# 烧录并运行（Ctrl+C 退出）
cargo run --locked

# 查看串口
python3 -m serial.tools.list_ports -v

# 找控制口
python3 host.py /dev/cu.usbmodemXXXX ping

# 设置变量
export CONTROL=/dev/cu.usbmodemXXXX
export UART=/dev/cu.usbmodemYYYY

# 验收
python3 host.py $CONTROL ping
python3 host.py $CONTROL swd
python3 host.py $CONTROL swd
python3 host.py $CONTROL ping
python3 host.py $UART uart-loopback

# 选做 CAN
python3 host.py $CONTROL can-send 123 11223344
python3 host.py $CONTROL can-recv

# 提交
git status
git add src/wiring.rs
git commit -m "Complete SWD GPIO and UART7 wiring"
git push
```

---

## 24. 附录 E：术语表

| 术语 | 一句话解释 |
| --- | --- |
| MCU | 集成 CPU、Flash、SRAM、外设的芯片 |
| Flash | 断电不丢的程序存储器 |
| SRAM | 运行时内存 |
| GPIO | 通用输入输出引脚 |
| AF | 引脚的复用功能编号 |
| Push-pull | 能主动拉高和拉低的输出 |
| Flex | 可在输入/输出间切换的引脚 |
| Pull-up | 内部上拉电阻 |
| UART | 异步串口 |
| Baud | 每秒传输的位数 |
| 8N1 | 8 数据位、无校验、1 停止位 |
| ISR | 中断服务函数 |
| Ring buffer | 首尾相接的缓冲区 |
| Overrun | 缓冲区满导致丢数据 |
| Loopback | TX 短接 RX 的自测 |
| USB endpoint | USB 单向数据通道 |
| Bulk | 大批量可重传的 USB 传输类型 |
| Packet | USB 一次传输的数据块 |
| Descriptor | USB 设备的自我介绍数据 |
| Enumeration | 主机识别 USB 设备的过程 |
| VID/PID | 厂商/产品标识 |
| CDC ACM | USB 虚拟串口类 |
| Composite | 一个 USB 设备包含多个功能 |
| WinUSB | Windows 用户态 USB 驱动 |
| DTR | 串口“主机已连接”信号 |
| CMSIS-DAP | Arm 的调试探针 USB 协议 |
| DAP_Info | 查询探针信息的命令 |
| DAP_Connect | 连接目标的命令 |
| SWD | 两线调试接口 |
| SWCLK | SWD 时钟 |
| SWDIO | SWD 双向数据 |
| DP | 调试端口 |
| AP | 访问端口 |
| DPIDR | 调试端口身份证寄存器 |
| ADIv5 | Arm 调试接口规范 v5 |
| Line reset | 连续高电平重置 SWD 总线 |
| JTAG-to-SWD | 用 0xE79E 切到 SWD 模式 |
| Turnaround | 交换驱动权的空时钟 |
| ACK | 目标的 3 位应答 |
| Parity | 奇偶校验位 |
| LSB first | 最低位先传 |
| no_std | 不使用 Rust 标准库 |
| panic-halt | panic 后停机 |
| Future | 可暂停的任务 |
| Executor | 轮询 future 的调度器 |
| join | 等所有 future 完成 |
| select | 等任意 future 完成 |
| StaticCell | 一次性初始化的静态存储 |
| Peri | 外设使用许可证 |
| bind_interrupts | 绑定中断和处理器 |
| heapless | 固定容量、不用堆的容器 |
| probe-rs | Rust 调试/烧录工具 |
| Runner | cargo run 实际执行的命令 |
| RTT | 通过 SWD 输出日志的通道 |
| ELF | 可执行文件格式 |
| Target triple | 编译目标三元组 |
| Cargo.lock | 依赖版本锁文件 |
| `--locked` | 严格按锁文件构建 |
| Linker script | 内存布局脚本 |
| OpenOCD | 另一套调试工具 |
| DFU | ST 系统 Bootloader 升级模式 |
| CAN | 控制器局域网 |
| Classical CAN | 经典 CAN，最多 8 字节 |
| CAN FD | 灵活数据速率 CAN |
| Transceiver | CAN 差分收发器 |
| tq | CAN 位时间的时间份额 |
| Sample point | CAN 采样点 |
| SJW | CAN 重新同步宽度 |
| Termination | CAN 终端电阻 |
| venv | Python 虚拟环境 |
| pyserial | Python 串口库 |

---

## 附：最短执行路径

如果你只想尽快完成，按这个顺序：

```sh
cd /Users/fangchuran/RoboMaster/硬件/培训/Embedded-test
source .venv/bin/activate
cargo build --locked
probe-rs list
cargo run --locked          # 烧录，完成后 Ctrl+C
# 拔探针、断电、接 SWD 跳线和 UART 回环跳线、上电
python3 -m serial.tools.list_ports -v
python3 host.py <控制口> ping
python3 host.py <控制口> swd
python3 host.py <控制口> swd
python3 host.py <控制口> ping
python3 host.py <UART口> uart-loopback
```

然后按 Step 13 填写引脚表和实测记录。
