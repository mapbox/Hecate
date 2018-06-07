<template>
    <div class='select-container'>
        <select v-model="panel" class='select select--s select--stroke'>
            <option>Deltas</option>
            <option>Bounds</option>
            <option>Styles</option>
        </select>
        <div class='select-arrow'></div>
    </div>
</template>

<script>
export default {
    name: 'panel',
    render: h => h(App),
    data: function() {
        return {
            panel: 'Deltas'
        };
    },
    watch: {
        panel: function() {
            this.$emit('input', this.panel);
        }
    },
}
</script>
