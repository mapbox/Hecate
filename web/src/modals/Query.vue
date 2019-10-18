<template>
<div class='absolute top left bottom right z3' style="pointer-events: none;">
    <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-events:auto;">
        <div class="flex-child px12 py12 w600 h400 bg-white round-ml shadow-darken10">
            <div class='grid w-full'>
                <div class='pb12 col col--12'>
                    <h3 class='fl txt-m txt-bold my6'>Database Query Tool</h3>

                    <button @click='close()' class='fr btn round bg-white color-black bg-darken25-on-hover h36'><svg class='icon'><use href='#icon-close'/></svg></button>

                    <div v-if='!error' class="flex-parent-inline fr mx12">
                        <button class="btn btn--pill btn--stroke btn--pill-hl round">Table Definitions</button>
                        <button @click='results = ""' class="btn btn--pill btn--stroke btn--pill-hc round">Query Editor</button>
                        <button @click='getQuery' class="btn btn--pill btn--stroke btn--pill-hr round">Results</button>
                    </div>
                </div>

                <template v-if='error'>
                    <div class='col col--12'>
                        <svg class='mx-auto icon h60 w60 color-red'><use xlink:href='#icon-alert'/></svg>
                    </div>
                    <div class='col col--12 align-center'>
                        <span>There was an error performing that query</span>
                    </div>
                    <div class='col col--12 align-center py24'>
                        <button @click='errorConfirm' class='btn btn--stroke round btn--red'>Ok</button>
                    </div>
                </template>
                <template v-else-if='!results.length'>
                    <div class='col col--12'>
                        <textarea id='editor' :readonly="!credentials.uid" class='textarea w-full h360' v-model="query" placeholder="Query SQL"></textarea>
                    </div>

                    <div class='col col--12'>
                        <p>Note the web UI only supports querying up to 100 features</p>
                    </div>
                    <div class='col col--12'>
                        <div class='grid grid--gut12'>
                            <div class='col col--8 py12'></div>
                            <div class='col col--4 py12'>
                                <button @click="getQuery" class='btn round btn--stroke w-full'>Query</button>
                            </div>
                        </div>
                    </div>
                </template>
                <template v-else>
                    <div class='col col--12'>
                        <textarea readonly class='textarea w-full h360' v-model="results" placeholder="SQL Results"></textarea>
                    </div>
                </template>
            </div>
        </div>
    </div>
</div>
</template>

<script>
import Behave from 'behave-js';

export default {
    name: 'query',
    data: function() {
        return {
            query: 'SELECT\n    *\nFROM\n    props',
            results: [],
            editor: false,
            error: false
        }
    },
    mounted: function() {
        this.editor = new Behave({
            textarea: document.getElementById('editor'),
            softTabs: false,
        });
    },
    methods: {
        close: function() {
            this.$emit('close');
        },
        errorConfirm: function() {
            this.results = '';
            this.error = false;
        },
        getQuery: function() {
            window.hecate.query.get(this.query, (err, results) => {
                if (err) return this.error = true;

                const EOF = results.pop();
                this.results = results;
            });
        }
    },
    render: h => h(App),
    props: ['credentials', 'auth']
}
</script>
