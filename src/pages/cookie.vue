<template>
    <el-dialog>
        <el-form
            v-model="form"
            ref="formRef"
            :rules="rules"
            label-width="100"
            size="small"
            inline
            width="100%"
        >
            <el-form-item placeholder="输入广场CK" prop="cookie">
                <el-input ref="input" v-model="form.cookie" />
            </el-form-item>
            <el-form-item>
                <el-button type="primary">保存</el-button>
            </el-form-item>
        </el-form>
        <el-table size="small" border>
            <el-table-column prop="idx" label="#" width="50" align="center" />
            <el-table-column prop="ck" label="内容" />
            <el-table-column label="操作" width="80" align="center" />
        </el-table>
    </el-dialog>
</template>

<script setup>
import { nextTick } from "vue";
import { onMounted, reactive, ref } from "vue";
let form = reactive({
    cookie: "",
});
let formRef = ref(null);
let input = ref(null);

const rules = {
    cookie: [{ required: true, trigger: "blur", message: "CK 不用为空" }],
};

const onSubmit = async () => {
    formRef.value.validate(async (valid) => {
        if (valid) {
            try {
            } catch (e) {
                console.log("exception for some add cookie?", e);
            }
        }
    });
};
</script>
