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

        onSubmit: () => {
          localStorage.setItem("username", $formData.username)
          localStorage.setItem("password", $formData.password)
        }
    })
    const { form: formData, enhance } = form

</script>

<div class="flex justify-center items-center w-full h-screen">
    <Card.Root class="sm:w-[400px] w-[70%]">
      <Card.Header>
        <Card.Title>Sign Up</Card.Title>
        <Card.Description>Create a new account</Card.Description>
      </Card.Header>
      <form class="" 
      method="POST" 
      use:enhance>
          <Card.Content>
              <Form.Field {form} class="mb-4" name="username">
                <Form.Control>
                  {#snippet children({ props })}
                    <Form.Label>Username</Form.Label>
                    <Input {...props} bind:value={$formData.username} placeholder="Your unique name" />
                  {/snippet}
                </Form.Control>
                <FormFieldErrors />
              </Form.Field>

              <Form.Field {form} class="mb-8" name="password">
                <Form.Control>
                  {#snippet children({ props })}
                    <Form.Label>Password</Form.Label>
                    <Input {...props} type="password" bind:value={$formData.password} placeholder="A strong password" />
                  {/snippet}
                </Form.Control>
        
              </Form.Field>
          </Card.Content>
          <Card.Footer class="flex justify-end">
              <div class="flex justify-between w-full">
                  <a href="/login" class="text-xs italic underline text-gray-200 h-full flex items-end pb-2">
                    Already have an account?
                  </a>
                  <Form.Button class="w-[80px]">
                      Signup
                  </Form.Button>
              </div>
          </Card.Footer>
        </form>
    </Card.Root>

</div>
