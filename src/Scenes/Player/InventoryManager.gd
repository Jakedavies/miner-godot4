extends Node
class_name InventoryManager

var inventory: Array[Item] = []

signal slot_updated(slot_id: int)

func aquire_item(new_item: Item, quantity: int):
	for slot_number in inventory.size():
		var item = inventory[slot_number]
		if item.name == new_item.name:
			item.count += quantity
			emit_slot_update(slot_number)
			return
	inventory[inventory.size()] = new_item
	emit_slot_update(inventory.size() - 1)

func use_item(slot_number: int):
	inventory[slot_number].use()
	emit_slot_update(slot_number)

func emit_slot_update(slot_number: int):
	slot_updated.emit(slot_number)
