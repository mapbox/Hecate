<template>
    <div v-if='modal_type === "login"' class='absolute top left bottom right z3' style="pointer-events: none;">
        <div class='flex-parent flex-parent--center-main flex-parent--center-cross h-full' style="pointer-evenents: auto;">
            <div class="flex-child px12 py12 w600 h300 bg-white round-ml shadow-darken10">
                <div class='grid w-full'>
                    <div class='col col--8'>
                        <h3 class='fl py6 txt-m txt-bold w-full'>Hecate Login</h3>
                    </div>
                    <div class='col col--4'>
                        <button @click='modal.type = false'class='fr btn round bg-white color-black bg-darken25-on-hover'><svg class='icon'><use href='#icon-close'/></svg></button>
                    </div>

                    <div class='col col--12 py12'>
                        <label>
                            Username:
                            <input v-model='modal.login.username' class='input py12' placeholder='username'/>
                        </label>
                    </div>

                    <div class='col col--12 py12'>
                        <label>
                            Password:
                            <input v-model='modal.login.password' type='password' class='input py12' placeholder='password'/>
                        </label>
                    </div>

                    <div class='col col--12'>
                        <div class='grid grid--gut12'>
                            <div class='col col--6 py12'>
                                <button @click='register_show' class='btn round bg-gray w-full'>Register</button>
                            </div>

                            <div class='col col--6 py12'>
                                <button @click='login' class='btn round w-full'>Login</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script>
export default {
    name: 'foot',
    render: h => h(App),
    props: ['modal_type']
}
</script>
