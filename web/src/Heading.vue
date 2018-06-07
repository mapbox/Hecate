<template>
    <div class='flex-child px12 py12'>
        <h3 class='fl py6 txt-m txt-bold' v-text='title'></h3>
        <button v-if='is_authed' @click='logout' class='fr px12 py12 btn round bg-white bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-logout'></use></svg></button>
        <button @click='login_show' class='fr px12 py12 btn round bg-white bg-darken25-on-hover color-gray-dark fr'><svg class='icon'><use href='#icon-user'></use></svg></button>
    </div>
</template>

<script>
export default {
    name: 'heading',
    render: h => h(App),
    props: [ 'is_authed', 'login_show', 'logout', 'title' ]
}
</script>
