## 总结实现的功能
实现sys_spawn，需要在 task中实现 spawn 函数，编写该函数时借鉴 fork 和 exec的过程，其需要像 fork那样创建新进程，但是不需要添加父子进程关系。stride 实现则是在tcb 中新增字段，并且task manager 中的 ready 队列从 FIFO 的双端队列修改为 vec，fetch 时选择stride最小的 task，并且更新task 值即可。
## 问答题
* 轮不到。P2的stride溢出后小于P1的stride
* stride的最大增量为BigStride / 2, stride算法选择stride最小的, 又因为prio的值大于等于2, 因此不考虑溢出的情况下最大值和最小值的差值小于等于BigStride / 2
```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let d = self.0 - other.0;
        if abs(d * 2) > BIG_STRIDE {
            if d < 0 {Some(Ordering::Greater)} else {Some(Ordering::Less)}
        } else {
            if d < 0 {Some(Ordering::Less)} else {Some(Ordering::Greater)}
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```
## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容： 无交流对象
2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：未参考资料
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
