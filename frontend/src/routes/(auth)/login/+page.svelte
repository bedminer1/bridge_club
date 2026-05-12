<script lang="ts">
    import * as Form from "$lib/components/ui/form/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import {
        type SuperValidated,
        type Infer,
        superForm,
    } from "sveltekit-superforms";
    import { userFormSchema, type UserFormSchema } from "../userSchema";
    import { zodClient } from "sveltekit-superforms/adapters";
    import FormFieldErrors from "$lib/components/ui/form/form-field-errors.svelte";

    let { data }: { data: { form: SuperValidated<Infer<UserFormSchema>> } } = $props()
    const form = superForm(data.form, {
        validators: zodClient(userFormSchema),
    })
    const { form: formData, enhance } = form

</script>

<div class="flex justify-center items-center w-full h-screen">
    <Card.Root class="sm:w-[400px] w-[70%]">
      <Card.Header>
        <Card.Title>Log In</Card.Title>
        <Card.Description>Sign in to your account</Card.Description>
      </Card.Header>
      <form class="" 
      method="POST" 
      use:enhance>
          <Card.Content>
              <Form.Field {form} class="mb-4" name="username">
                <Form.Control>
                  {#snippet children({ props })}
                    <Form.Label>Username</Form.Label>
                    <Input {...props} bind:value={$formData.username} />
                  {/snippet}
                </Form.Control>
                <FormFieldErrors />
              </Form.Field>

              <Form.Field {form} class="mb-8" name="password">
                <Form.Control>
                  {#snippet children({ props })}
                    <Form.Label>Password</Form.Label>
                    <Input {...props} type="password" bind:value={$formData.password} />
                  {/snippet}
                </Form.Control>
                <FormFieldErrors />
              </Form.Field>
          </Card.Content>
          <Card.Footer class="flex justify-end">
               <div class="flex justify-between w-full">
                    <a href="/signup" class="text-xs italic underline text-gray-200 h-full flex items-end pb-2">
                        Don't have an account?
                    </a>
                    <Form.Button class="w-[80px]">
                        Login
                    </Form.Button>
                </div>
          </Card.Footer>
        </form>
    </Card.Root>

</div>

<!-- <div class="w-[320px]">
    <div>
    <h1 class="text-center mb-2">Login</h1>
    <form 
        class="flex flex-col gap-2 mb-2"
        onsubmit={login}>
        <div class="flex justify-between gap-4">
            <Label class="w-[60px] pr-6" for="username">Username</Label>
            <Input bind:value={username} />
        </div>
        <div class="flex justify-between gap-4 mb-8">
            <Label class="w-[60px] pr-6" for="password">Password</Label>
            <Input type="password" bind:value={password} />
        </div>
        <div class="flex justify-between">
            <a href="/signup" class="text-xs italic underline text-gray-200 h-full flex items-end pb-2">Not Signed Up?</a>
            <Form.Button class="w-[80px]">
                Login
            </Form.Button>
        </div>
    </form>
    </div>
</div> -->